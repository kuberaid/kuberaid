use std::{
    net::SocketAddr,
    ops::Deref,
    pin::Pin,
    sync::{Arc, RwLock},
    time::Duration,
};

use kube::Client;
use kuberaid_api::v1::{
    GetPoolRequest, GetPoolResponse, ListPoolsRequest, ListPoolsResponse, Pool, State,
    WatchRequest, WatchResponse,
    manager_server::{Manager, ManagerServer},
};
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};
use tracing::info;
use zfs::{
    ZfsBackend,
    cli::{ZfsCli, ZfsEvent},
    new::Zfs,
};

use crate::manager::controller::{pool, storagenode};

pub mod controller;

pub struct KuberaidManagerInner {
    node_name: String,

    client: Client,
    zfs: zfs::new::Zfs,
}

#[derive(Clone)]
pub struct KuberaidManager(Arc<KuberaidManagerInner>);

impl Deref for KuberaidManager {
    type Target = KuberaidManagerInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl KuberaidManager {
    pub async fn new(node_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        assert!(!node_name.is_empty());
        let client = Client::try_default().await?;

        Ok(Self(Arc::new(KuberaidManagerInner {
            node_name,
            client,
            zfs: Zfs::new().await?,
        })))
    }

    pub async fn run(self, addr: SocketAddr) {
        let _ = tokio::join!(
            self.clone().serve(addr),
            storagenode::run(self.clone()),
            pool::run(self)
        );
    }

    async fn serve(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        Server::builder()
            .add_service(ManagerServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

type WatchResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<WatchResponse, Status>> + Send>>;

#[tonic::async_trait]
impl Manager for KuberaidManager {
    type WatchStateStream = ResponseStream;

    async fn watch_state(&self, req: Request<WatchRequest>) -> WatchResult<Self::WatchStateStream> {
        info!("\tclient connected from: {:?}", req.remote_addr());
        let mut stream = zfs::cli::ZfsCli::events()
            .unwrap()
            .map(|item| WatchResponse {
                message: format!("{item:?}"),
            });

        // spawn and channel are required if you want handle "disconnect" functionality
        // the `out_stream` will not be polled after client disconnect
        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        // item (server response) was queued to be send to client
                    }
                    Err(_item) => {
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
            }
            info!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::WatchStateStream
        ))
    }

    async fn list_pools(
        &self,
        request: Request<ListPoolsRequest>,
    ) -> Result<Response<ListPoolsResponse>, Status> {
        info!("Got a request: {request:?}");

        let pools = ZfsCli::pools()
            .await
            .map_err(|e| Status::internal(format!("{e}")))?
            .into_values()
            .map(|p| Pool {
                name: p.name,
                state: match p.inner.state {
                    _ => State::Online as i32,
                },
            })
            .collect();

        let reply = ListPoolsResponse { pools };

        Ok(Response::new(reply))
    }

    async fn get_pool(
        &self,
        request: Request<GetPoolRequest>,
    ) -> Result<Response<GetPoolResponse>, Status> {
        info!("Got a request: {request:?}");

        let req = request.into_inner();

        let pool = ZfsCli::pools()
            .await
            .map_err(|e| Status::internal(format!("{e}")))?
            .into_values()
            .find(|p| p.name.eq(&req.name))
            .map(|p| Pool {
                name: p.name,
                state: match p.inner.state {
                    _ => State::Online as i32,
                },
            });

        let reply = GetPoolResponse { pool };

        Ok(Response::new(reply))
    }
}
