use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;
use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, TimeDelta, Utc};
use futures::StreamExt;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio::task::JoinHandle;
use tracing::info;

use crate::cli::{EventStream, HistoryEventKind, ZfsCli, ZfsEvent, ZfsEventKind, ZfsScalar};
use crate::{Property, Result, ZDataset, ZPool, ZfsBackend};

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct DatasetId(String);
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PoolId(u64);

#[derive(Clone, Debug)]
pub struct Dataset {
    // pub id: DatasetId,
    pub name: String,
    pub path: PathBuf,
    pub parent: Option<DatasetId>,
    pub children: Vec<DatasetId>,

    pub pool: PoolId,

    pub properties: HashMap<String, Property<ZfsScalar>>,
}

#[derive(Debug, Clone, Default)]
pub struct Pool {
    pub guid: PoolId,
    pub name: String,

    pub datasets: Vec<DatasetId>,

    pub destroyed: bool,
    pub properties: HashMap<String, Property<ZfsScalar>>,
}

#[derive(Default, Debug)]
pub struct DatasetTree {
    datasets: RwLock<HashMap<DatasetId, Dataset>>,
    pools: RwLock<HashMap<PoolId, Pool>>,
}

impl DatasetTree {
    async fn set_datasets(&self, d: Vec<ZDataset>) {
        let mut datasets = self.datasets.write().await;

        datasets.clear();
        for dataset in d {
            datasets.insert(
                DatasetId(dataset.name.clone()),
                Dataset {
                    name: dataset.name.clone(),
                    path: PathBuf::from(dataset.name),
                    parent: None,
                    children: Vec::new(),
                    pool: PoolId(0),
                    properties: dataset.properties,
                },
            );
        }
    }

    async fn set_pools(&self, p: Vec<ZPool>) {
        let mut pools = self.pools.write().await;

        pools.clear();
        for pool in p {
            let guid = PoolId(pool.inner.pool_guid);
            pools.insert(
                guid,
                Pool {
                    guid,
                    name: pool.name,
                    properties: pool.properties,
                    ..Pool::default()
                },
            );
        }
    }

    async fn delta_dataset(&self, id: DatasetId, dataset: Option<ZDataset>) {
        let mut datasets = self.datasets.write().await;
        if let Some(dataset) = dataset {
            let d = datasets.entry(id).or_insert(Dataset {
                properties: HashMap::with_capacity(dataset.properties.capacity()),
                name: dataset.name,
                path: PathBuf::new(),
                parent: None,
                children: vec![],
                pool: PoolId::default(),
            });
            d.properties.extend(dataset.properties);
        } else {
            datasets.remove(&id);
        }
    }

    async fn delta_pool(&self, id: PoolId, pool: Option<ZPool>, destroyed: bool) {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pool {
            let p = pools.entry(id).or_insert(Pool {
                guid: id,
                name: pool.name,
                datasets: vec![],
                destroyed,
                properties: HashMap::with_capacity(pool.properties.capacity()),
            });
            p.properties.extend(pool.properties);
            p.destroyed = destroyed;
        } else if !destroyed {
            pools.remove(&id);
        } else if destroyed {
            let Some(p) = pools.get_mut(&id) else {
                return;
            };
            p.destroyed = destroyed;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Zfs(Arc<ZfsInner>);

#[derive(Debug)]
pub struct ZfsInner {
    tree: Arc<DatasetTree>,
    events: broadcast::Sender<ZfsEvent>,
    watcher: JoinHandle<Result<()>>,
}

impl Deref for Zfs {
    type Target = ZfsInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ZfsInner {
    fn new() -> Self {
        let tree: Arc<DatasetTree> = Arc::default();

        let tx = tokio::sync::broadcast::Sender::new(5);
        let watcher = tokio::spawn({
            let tree = tree.clone();
            let tx = tx.clone();
            async move {
                loop {
                    let now = Utc::now();
                    let mut events = ZfsCli::events()?;
                    while let Some(event) = events.next().await {
                        if now > event.time {
                            continue;
                        }

                        match &event.kind {
                            ZfsEventKind::HistoryEvent {
                                hostname,
                                txg,
                                kind,
                            } => match kind {
                                HistoryEventKind::Set {
                                    property,
                                    value,
                                    dataset: Some(dataset),
                                } => {
                                    if let Ok(d) =
                                        ZfsCli::get_dataset_with_prop(&dataset.name, property).await
                                    {
                                        tree.delta_dataset(DatasetId(dataset.name.clone()), d)
                                            .await;
                                    }
                                }
                                HistoryEventKind::Set {
                                    property,
                                    value,
                                    dataset: None,
                                } => {
                                    if let Ok(p) =
                                        ZfsCli::get_pool_with_prop(&event.pool, property).await
                                    {
                                        tree.delta_pool(PoolId(event.pool_guid), p, false).await;
                                    }
                                }

                                HistoryEventKind::Import => {}
                                HistoryEventKind::Open => {}

                                _ => {}
                            },

                            ZfsEventKind::PoolImport => {
                                if let Ok(p) = ZfsCli::get_pool(&event.pool).await {
                                    tree.delta_pool(PoolId(event.pool_guid), p, false).await;
                                }
                            }
                            ZfsEventKind::PoolExport => {
                                tree.delta_pool(PoolId(event.pool_guid), None, false).await;
                            }

                            ZfsEventKind::PoolDestroy => {
                                tree.delta_pool(PoolId(event.pool_guid), None, true).await;
                            }

                            ZfsEventKind::ConfigSync => {}

                            _ => {}
                        }

                        let _ = tx.send(event);
                    }
                }
            }
        });

        Self {
            tree,
            watcher,
            events: tx,
        }
    }
}

impl Zfs {
    pub async fn new() -> Result<Self> {
        let mut s = Self(Arc::new(ZfsInner::new()));
        s.refresh().await?;
        Ok(s)
    }

    pub async fn refresh(&self) -> Result<()> {
        let datasets = ZfsCli::datasets().await?;
        let pools = ZfsCli::pools().await?;

        self.tree.set_pools(pools.into_values().collect()).await;
        self.tree
            .set_datasets(datasets.into_values().collect())
            .await;

        Ok(())
    }

    pub async fn get_pools(&self) -> Vec<Pool> {
        self.tree.pools.read().await.values().cloned().collect()
    }

    pub async fn get_pool(&self, name: &str) -> Option<Pool> {
        self.tree
            .pools
            .read()
            .await
            .values()
            .find(|v| v.name.eq(name))
            .cloned()
    }

    pub async fn get_dataset(&self, name: &str) -> Option<Dataset> {
        self.tree
            .datasets
            .read()
            .await
            .get(&DatasetId(name.to_string()))
            .cloned()
    }

    #[must_use]
    pub fn events(&self) -> broadcast::Receiver<ZfsEvent> {
        self.events.subscribe()
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    pub async fn it_works() -> Result<()> {
        let zfs = Zfs::new().await?;

        Ok(())
    }
}
