#![warn(clippy::pedantic)]

use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use kube::Client;
use kuberaid_manager::{
    csi::CSIPlugin,
    manager::{KuberaidManager, controller},
};
use nix::unistd::gethostname;

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
    Manager {
        #[arg(short, long, env = "BIND", default_value = "[::]:50051")]
        address: SocketAddr,

        #[arg(short, long, env = "NODE_NAME", default_value_t = gethostname().ok().and_then(|o| o.into_string().ok()).unwrap_or_default())]
        node_name: String,
    },
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
            csi.serve(endpoint).await?;
        }
        Some(Commands::Manager { address, node_name }) => {
            let manager = KuberaidManager::new(node_name.clone()).await?;

            manager.run(*address).await;
        }
        None => {}
    }

    Ok(())
}
