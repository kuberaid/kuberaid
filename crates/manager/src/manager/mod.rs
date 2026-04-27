use std::{
    pin::Pin,
    sync::{Arc, RwLock},
    time::Duration,
};

use kuberaid_common::grpc::{WatchRequest, WatchResponse, agent_server::Agent};
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct KuberaidManagerInner {
    zfs: zfs::new::Zfs,
    // healthy: RwLock<(bool, bool)>,
}

#[derive(Debug, Default, Clone)]
pub struct KuberaidManager(Arc<KuberaidManagerInner>);

type WatchResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<WatchResponse, Status>> + Send>>;

#[tonic::async_trait]
impl Agent for KuberaidManager {
    type WatchStateStream = ResponseStream;

    async fn watch_state(&self, req: Request<WatchRequest>) -> WatchResult<Self::WatchStateStream> {
        println!("EchoServer::server_streaming_echo");
        println!("\tclient connected from: {:?}", req.remote_addr());

        let repeat = std::iter::repeat(WatchResponse {
            message: "test".to_string(),
        });
        let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));

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
            println!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::WatchStateStream
        ))
    }

    //     async fn list_pools(
    //         &self,
    //         request: Request<ListPoolsRequest>,
    //     ) -> Result<Response<ListPoolsResponse>, Status> {
    //         println!("Got a request: {request:?}");

    //         let pools = zfs::ZfsCli::pools()
    //             .await
    //             .map_err(|e| Status::internal(format!("{e}")))?
    //             .into_values()
    //             .map(|p| Pool {
    //                 name: p.name,
    //                 state: match p.inner.state {
    //                     _ => State::Online as i32,
    //                 },
    //             })
    //             .collect();

    //         let reply = ListPoolsResponse { pools };

    //         Ok(Response::new(reply))
    //     }

    //     async fn get_pool(
    //         &self,
    //         request: Request<GetPoolRequest>,
    //     ) -> Result<Response<GetPoolResponse>, Status> {
    //         println!("Got a request: {request:?}");

    //         let req = request.into_inner();

    //         let pool = zfs::ZfsCli::get_pool(&req.name)
    //             .await
    //             .map_err(|e| Status::internal(format!("{e}")))?
    //             .map(|p| Pool {
    //                 name: p.name,
    //                 state: match p.inner.state {
    //                     _ => State::Online as i32,
    //                 },
    //             });

    //         let reply = GetPoolResponse { pool };

    //         Ok(Response::new(reply))
    //     }
}
