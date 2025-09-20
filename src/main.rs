//! Olympus blockchain node main entry point

use clap::{Parser, Subcommand};
use olympus::core::config::Config;
use olympus::Result;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "olympus")]
#[command(about = "Olympus blockchain node implementation in Rust")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the node
    Start {
        /// Data directory path
        #[arg(long, default_value = "./data")]
        data_path: PathBuf,
        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,
        /// Run as witness node
        #[arg(long)]
        witness: bool,
        /// Witness account keystore file
        #[arg(long)]
        witness_account: Option<PathBuf>,
        /// Password for keystore file
        #[arg(long)]
        password: Option<String>,
    },
    /// Initialize configuration
    Init {
        /// Output configuration file path
        #[arg(long, default_value = "config.toml")]
        output: PathBuf,
    },
    /// Show node version and information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            data_path,
            config,
            witness,
            witness_account,
            password,
        } => {
            info!("Starting Olympus node...");
            start_node(data_path, config, witness, witness_account, password).await?;
        }
        Commands::Init { output } => {
            info!("Initializing configuration...");
            init_config(output)?;
        }
        Commands::Version => {
            println!("Olympus Rust Implementation v{}", env!("CARGO_PKG_VERSION"));
            println!("Chain ID: {}", olympus::core::types::CHAIN_ID);
        }
    }

    Ok(())
}

async fn start_node(
    data_path: PathBuf,
    config_path: Option<PathBuf>,
    _witness: bool,
    _witness_account: Option<PathBuf>,
    _password: Option<String>,
) -> Result<()> {
    // Load configuration
    let config = if let Some(path) = config_path {
        Config::load_from_file(path)
            .map_err(|e| olympus::OlympusError::Serialization(e.to_string()))?
    } else {
        Config::default()
    };

    info!("Configuration loaded");
    info!("Data path: {:?}", data_path);
    info!("Network: {}:{}", config.network.listen_address, config.network.listen_port);
    info!("RPC: {}:{}", config.rpc.listen_address, config.rpc.listen_port);

    // Create data directory if it doesn't exist
    if !data_path.exists() {
        std::fs::create_dir_all(&data_path)
            .map_err(|e| olympus::OlympusError::Database(e.to_string()))?;
        info!("Created data directory: {:?}", data_path);
    }

    // TODO: Initialize database
    info!("Initializing database...");

    // TODO: Initialize P2P network
    info!("Initializing P2P network...");

    // TODO: Initialize consensus engine
    info!("Initializing consensus engine...");

    // TODO: Initialize RPC server
    if config.rpc.enabled {
        info!("Starting RPC server...");
    }

    // TODO: Start witness mode if enabled
    if _witness {
        info!("Starting in witness mode...");
    }

    info!("Olympus node started successfully!");

    // Keep the node running
    tokio::signal::ctrl_c().await
        .map_err(|e| olympus::OlympusError::Network(e.to_string()))?;

    info!("Shutting down Olympus node...");
    Ok(())
}

fn init_config(output: PathBuf) -> Result<()> {
    Config::create_default_config(output)
        .map_err(|e| olympus::OlympusError::Serialization(e.to_string()))?;
    
    println!("Configuration file created successfully!");
    Ok(())
}
