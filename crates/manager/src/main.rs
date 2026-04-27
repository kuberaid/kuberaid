#![warn(clippy::pedantic)]

use std::path::PathBuf;

use k8s_csi::v1::{
    controller_server::ControllerServer, identity_server::IdentityServer, node_server::NodeServer,
};
use kuberaid_common::grpc::agent_server::AgentServer;
use kuberaid_manager::{csi::CSIPlugin, manager::KuberaidManager};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Csi {
        #[arg(
            short,
            long,
            env = "CSI_ENDPOINT",
            default_value = "unix:///csi/csi.sock"
        )]
        endpoint: PathBuf,
    },
    Manager,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Csi { endpoint }) => {
            let csi = CSIPlugin::default();

            let uds = UnixListener::bind(endpoint)?;
            let incoming = UnixListenerStream::new(uds);

            Server::builder()
                .add_service(IdentityServer::new(csi.clone()))
                .add_service(ControllerServer::new(csi.clone()))
                .add_service(NodeServer::new(csi))
                .serve_with_incoming(incoming)
                .await?;
        }
        Some(Commands::Manager) => {
            let addr = "[::1]:50051".parse()?;
            let manager = KuberaidManager::default();

            Server::builder()
                .add_service(AgentServer::new(manager.clone()))
                .serve(addr)
                .await?;
        }
        None => {}
    }

    Ok(())
}
