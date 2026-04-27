use std::sync::Arc;
use std::time::Instant;
use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, TimeDelta, Utc};
use futures::StreamExt;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::cli::{EventStream, HistoryEventKind, ZfsCli, ZfsEvent, ZfsEventKind, ZfsScalar};
use crate::{Property, Result, ZDataset, ZPool, ZfsBackend};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct DatasetId(String);
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Pool {
    pub guid: PoolId,
    pub name: String,

    pub datasets: Vec<DatasetId>,

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
                    datasets: Vec::new(),
                    properties: pool.properties,
                },
            );
        }
    }

    async fn delta_dataset(&self, id: &DatasetId, dataset: ZDataset) {
        let mut datasets = self.datasets.write().await;
        if let Some(d) = datasets.get_mut(id) {
            d.properties.extend(dataset.properties);
        }
    }
}

#[derive(Debug)]
pub struct Zfs {
    tree: Arc<DatasetTree>,
    // events: EventStream,
    watcher: JoinHandle<Result<()>>,
}

impl Default for Zfs {
    fn default() -> Self {
        let tree: Arc<DatasetTree> = Arc::default();

        let watcher = tokio::spawn({
            let tree = tree.clone();
            async move {
                loop {
                    let now = Utc::now();
                    let mut events = ZfsCli::events()?;
                    while let Some(event) = events.next().await {
                        if event.time < now {
                            continue;
                        }

                        match event.kind {
                            ZfsEventKind::HistoryEvent {
                                hostname,
                                txg,
                                kind,
                            } => match kind {
                                HistoryEventKind::Set {
                                    dataset_id,
                                    dataset_name,
                                    property,
                                    value,
                                } => {
                                    if let Ok(Some(d)) =
                                        ZfsCli::get_dataset_with_prop(&dataset_name, &property)
                                            .await
                                    {
                                        tree.delta_dataset(&DatasetId(dataset_name), d).await;
                                    }
                                }
                                HistoryEventKind::Import => {}
                                HistoryEventKind::Open => {}

                                _ => {}
                            },

                            ZfsEventKind::ConfigSync => {}

                            _ => {}
                        }
                    }
                }
            }
        });

        Self { tree, watcher }
    }
}

impl Zfs {
    pub async fn new() -> Result<Self> {
        let mut s = Self::default();
        s.refresh().await?;
        Ok(s)
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let datasets = ZfsCli::datasets().await?;
        let pools = ZfsCli::pools().await?;

        self.tree.set_pools(pools.into_values().collect()).await;
        self.tree
            .set_datasets(datasets.into_values().collect())
            .await;

        Ok(())
    }

    pub async fn get_dataset(&self, name: &str) -> Option<Dataset> {
        self.tree
            .datasets
            .read()
            .await
            .get(&DatasetId(name.to_string()))
            .cloned()
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
