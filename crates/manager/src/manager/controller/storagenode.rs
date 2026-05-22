use std::{sync::Arc, time::Duration};

use kube::{
    Api, Error, Resource, ResourceExt,
    api::{ObjectMeta, Patch, PatchParams, PostParams},
    runtime::{
        Controller, WatchStreamExt, controller::Action, reflector::ObjectRef, watcher::Config,
    },
};
use tokio_stream::{StreamExt as TokioStreamExt, wrappers::BroadcastStream};

use crate::{
    crds::{Pool, StorageNode},
    manager::KuberaidManager,
};
use tracing::{error, info};

use crate::NAMESPACE;

// struct StorageNodeController;

// impl StorageNodeController {
//     pub fn new() {
//         let
//     }

async fn reconcile(
    object: Arc<StorageNode>,
    manager: Arc<KuberaidManager>,
) -> Result<Action, Error> {
    let pools = Api::<Pool>::all(manager.client.clone());
    let storagenode = Api::<StorageNode>::all(manager.client.clone())
        .get(&manager.node_name)
        .await?;
    let oref = storagenode.controller_owner_ref(&()).unwrap();
    info!("Reconciling");

    let serverside = PatchParams::apply(&format!("storagenode.{NAMESPACE}"));
    for pool in manager.zfs.get_pools().await {
        if pool.destroyed {
            continue;
        }

        let obj = &Pool {
            metadata: ObjectMeta {
                name: Some(pool.name.clone()),
                owner_references: Some(vec![oref.clone()]),
                ..ObjectMeta::default()
            },
            ..Pool::default()
        };

        pools
            .patch(&obj.name_any(), &serverside, &Patch::Apply(obj))
            .await?;
    }

    Ok(Action::await_change())
}
// }

fn error_policy(_object: Arc<StorageNode>, err: &Error, _ctx: Arc<KuberaidManager>) -> Action {
    error!("{err}");
    Action::requeue(Duration::from_secs(5))
}

pub async fn run(manager: KuberaidManager) -> Result<(), kube::Error> {
    let storagenodes = Api::<StorageNode>::all(manager.client.clone());
    // let pools = Api::<Pool>::all(client);

    let (reader, writer) = kube::runtime::reflector::store();
    let stream = kube::runtime::reflector(
        writer,
        kube::runtime::watcher(storagenodes.clone(), Config::default()),
    )
    .default_backoff()
    .applied_objects()
    .filter({
        let manager = manager.clone();
        move |o| {
            o.as_ref().is_ok_and(|o| {
                o.metadata
                    .name
                    .as_ref()
                    .is_some_and(|n| n.eq(&manager.node_name))
            })
        }
    });

    let zfs_stream = BroadcastStream::new(manager.zfs.events())
        .filter_map(|e| e.ok())
        .map({
            let manager = manager.clone();
            move |e| ObjectRef::new(&manager.node_name)
        });

    let mut stream = Box::pin(
        Controller::for_stream(stream, reader)
            .owns(Api::<Pool>::all(manager.client.clone()), Config::default())
            .reconcile_on(zfs_stream)
            .run(reconcile, error_policy, Arc::new(manager)),
    );

    while let Some(v) = stream.next().await {
        match v {
            Ok(_) => {}
            // Err(watcher::Error::) => {}
            Err(kube::runtime::controller::Error::ObjectNotFound(o)) => {
                let _ = storagenodes
                    .create(
                        &PostParams::default(),
                        &StorageNode {
                            metadata: ObjectMeta {
                                name: Some(o.name),
                                ..ObjectMeta::default()
                            },
                            spec: Default::default(),
                            status: Default::default(),
                        },
                    )
                    .await;
            }
            Err(e) => error!("{e}"),
            // Err(e) => match e {
            //     kube::runtime::controller::Error::ObjectNotFound(o) => todo!(),
            //     kube::runtime::controller::Error::ReconcilerFailed(_, object_ref) => todo!(),
            //     kube::runtime::controller::Error::QueueError(_) => todo!(),
            //     kube::runtime::controller::Error::RunnerError(error) => todo!(),
            // },
        }
    }

    Ok(())
}
