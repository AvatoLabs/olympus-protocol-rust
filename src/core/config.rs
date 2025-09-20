//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Olympus node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Data directory path
    pub data_path: PathBuf,
    /// Network configuration
    pub network: NetworkConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// RPC configuration
    pub rpc: RpcConfig,
    /// Consensus configuration
    pub consensus: ConsensusConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_address: String,
    /// Listen port
    pub listen_port: u16,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Maximum number of peers
    pub max_peers: usize,
    /// Enable UPnP
    pub enable_upnp: bool,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Cache size in MB
    pub cache_size: u64,
    /// Write buffer size in MB
    pub write_buffer_size: u64,
    /// Enable cache filter
    pub cache_filter: bool,
}

/// RPC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// Enable RPC server
    pub enabled: bool,
    /// RPC listen address
    pub listen_address: String,
    /// RPC listen port
    pub listen_port: u16,
    /// Enable WebSocket
    pub enable_websocket: bool,
    /// CORS origins
    pub cors_origins: Vec<String>,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum witnesses required
    pub min_witnesses: u64,
    /// Maximum witnesses
    pub max_witnesses: u64,
    /// Epoch period (number of blocks)
    pub epoch_period: u64,
    /// Gas limit per block
    pub gas_limit: u64,
    /// Gas price in wei
    pub gas_price: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log file path
    pub file: Option<PathBuf>,
    /// Enable console logging
    pub console: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from("./data"),
            network: NetworkConfig::default(),
            database: DatabaseConfig::default(),
            rpc: RpcConfig::default(),
            consensus: ConsensusConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0".to_string(),
            listen_port: 30607,
            bootstrap_nodes: vec![],
            max_peers: 50,
            enable_upnp: true,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            cache_size: 2048, // 2GB
            write_buffer_size: 256, // 256MB
            cache_filter: true,
        }
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_address: "127.0.0.1".to_string(),
            listen_port: 8765,
            enable_websocket: true,
            cors_origins: vec!["*".to_string()],
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_witnesses: 7,
            max_witnesses: 14,
            epoch_period: 10000,
            gas_limit: 50_000_000,
            gas_price: 10_000_000,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            console: true,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Create default configuration file
    pub fn create_default_config<P: AsRef<std::path::Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        config.save_to_file(path)?;
        Ok(())
    }
}
