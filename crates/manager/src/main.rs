#![warn(clippy::pedantic)]

use std::{net::SocketAddr, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use nix::unistd::gethostname;

use kuberaid_manager::{csi::CSIPlugin, manager::KuberaidManager};

#[derive(Subcommand)]
pub enum Commands {
    Csi {
        #[arg(short, long, env = "CSI_ENDPOINT", default_value = "/csi/csi.sock")]
        endpoint: PathBuf,
    },
    Manager {
        #[arg(short, long, env = "BIND", default_value = "[::]:50051")]
        address: SocketAddr,
    },
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, env = "NODE_NAME", default_value_t = gethostname().ok().and_then(|o| o.into_string().ok()).unwrap_or_default())]
    node_name: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Csi { endpoint }) => {
            let csi = CSIPlugin::new(cli.node_name);
            csi.serve(endpoint).await?;
        }
        Some(Commands::Manager { address }) => {
            let manager = KuberaidManager::new(cli.node_name).await?;

            manager.run(*address).await;
        }
        None => {}
    }

    Ok(())
}
