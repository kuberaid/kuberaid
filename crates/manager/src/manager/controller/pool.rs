use std::{sync::Arc, time::Duration};

// use futures::StreamExt;
use kube::{
    Api, Client, Error, Resource, ResourceExt,
    api::{DeleteParams, ObjectMeta, Patch, PatchParams, PostParams},
    runtime::{
        Controller, WatchStreamExt,
        controller::Action,
        reflector::{Lookup, ObjectRef},
        watcher::{self, Config},
    },
};
use kuberaid_api::v1::manager_server::Manager;
use serde_json::json;
use tokio_stream::{StreamExt as TokioStreamExt, wrappers::BroadcastStream};
use zfs::cli::ZfsScalar;

use crate::{
    crds::{Pool, PoolStatus, StorageNode},
    manager::KuberaidManager,
};
use tracing::{error, info};

async fn reconcile(obj: Arc<Pool>, manager: Arc<KuberaidManager>) -> Result<Action, Error> {
    // let _ = manager.zfs.refresh().await;
    let status = if let Some(pool) = manager.zfs.get_pool(&obj.name_any()).await {
        info!("{pool:?}");
        if pool.destroyed {
            None
        } else {
            Some(PoolStatus {
                imported: true,
                properties: pool
                    .properties
                    .into_iter()
                    .map(|(k, v)| match v.value {
                        ZfsScalar::F32(f) => (k, f.to_string()),
                        ZfsScalar::U64(u) => (k, u.to_string()),
                        ZfsScalar::String(s) => (k, s),
                        _ => (k, String::new()),
                    })
                    .collect(),
                ..PoolStatus::default()
            })
        }
    } else {
        Some(PoolStatus {
            imported: false,
            ..PoolStatus::default()
        })
    };

    let pools = Api::<Pool>::all(manager.client.clone());
    if let Some(status) = status {
        let status = json!({ "status": status });

        pools
            .patch_status(
                &obj.name_any(),
                &PatchParams::default(),
                &Patch::Merge(&status),
            )
            .await?;
    } else {
        info!("Delet");
        pools
            .delete(&obj.name_any(), &DeleteParams::foreground())
            .await?;
    }

    Ok(Action::await_change())
}

fn error_policy(_object: Arc<Pool>, _err: &Error, _ctx: Arc<KuberaidManager>) -> Action {
    Action::requeue(Duration::from_secs(5))
}

pub async fn run(manager: KuberaidManager) -> Result<(), kube::Error> {
    let pools = Api::<Pool>::all(manager.client.clone());
    let storagenode = Api::<StorageNode>::all(manager.client.clone())
        .get(&manager.node_name)
        .await?;
    let oref = storagenode.controller_owner_ref(&()).unwrap();

    let (reader, writer) = kube::runtime::reflector::store();
    let stream = kube::runtime::reflector(
        writer,
        kube::runtime::watcher(pools.clone(), Config::default()),
    )
    .default_backoff()
    .applied_objects()
    .filter(move |o| {
        o.as_ref()
            .is_ok_and(|o| o.owner_references().contains(&oref))
    });

    let zfs_stream = BroadcastStream::new(manager.zfs.events())
        .filter_map(|e| e.ok())
        .map(|e| ObjectRef::new(&e.pool));

    let mut stream = Box::pin(
        Controller::for_stream(stream, reader)
            // Controller::new(pools.clone(), Config::default())
            .reconcile_on(zfs_stream)
            .run(reconcile, error_policy, Arc::new(manager)),
    );

    while let Some(v) = stream.next().await {
        match v {
            Ok(_) => {}
            Err(e) => error!("{e}"),
        }
    }

    Ok(())
}
