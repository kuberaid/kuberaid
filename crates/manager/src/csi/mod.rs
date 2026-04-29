use std::path::PathBuf;

use k8s_csi::v1::{
    controller_server::ControllerServer, identity_server::IdentityServer, node_server::NodeServer,
};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

mod controller;
mod identity;
mod node;

#[derive(Debug, Default, Clone)]
pub struct CSIPlugin;

impl CSIPlugin {
    pub async fn serve(self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let uds = UnixListener::bind(path)?;
        let incoming = UnixListenerStream::new(uds);

        Server::builder()
            .add_service(IdentityServer::new(self.clone()))
            .add_service(ControllerServer::new(self.clone()))
            .add_service(NodeServer::new(self))
            .serve_with_incoming(incoming)
            .await?;

        Ok(())
    }
}
