use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QsbConfig {
    pub node: NodeConfig,
    pub networking: NetworkingConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    pub name: String,
    pub listen_addr: std::net::SocketAddr,
    pub data_dir: std::path::PathBuf,
    pub genesis: GenesisConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub timestamp: u64,
    pub difficulty: u32,
    pub initial_supply: u64,
    pub genesis_message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkingConfig {
    pub max_peers: usize,
    pub discovery_interval: std::time::Duration,
    pub connection_timeout: std::time::Duration,
    pub bootstrap_peers: Vec<std::net::SocketAddr>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackendType,
    pub path: std::path::PathBuf,
    pub cache_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StorageBackendType {
    RocksDb,
    Sled,
    InMemory,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(std::path::PathBuf),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub metrics_addr: Option<std::net::SocketAddr>,
    pub export_interval: std::time::Duration,
}
