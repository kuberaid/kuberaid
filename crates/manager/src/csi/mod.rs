use std::path::PathBuf;

use anyhow::Result;
use k8s_csi::v1::{
    controller_server::ControllerServer, identity_server::IdentityServer, node_server::NodeServer,
};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use tracing::warn;

mod controller;
mod identity;
mod node;

#[derive(Debug, Clone)]
pub struct CSIPlugin {
    node_name: String,
}

impl CSIPlugin {
    pub fn new(node_name: String) -> Self {
        Self { node_name }
    }

    pub async fn serve(self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            warn!("Cleaning up existing socket: {}", path.display());
            tokio::fs::remove_file(path).await?;
        }
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
