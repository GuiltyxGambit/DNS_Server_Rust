use std::{array, net::{IpAddr, SocketAddr}, path::PathBuf};
use core::str;
use serde::Deserialize;

#[derive(Deserialize)]
struct FileConfig {
    pub server: ServerConfig,
    pub tcp: Option<TcpConfig>,
    pub logging: Option<LoggingConfig>,
    pub recursion: Option<RecursionConfig>,
    pub zones: Vec<ZoneConfig>
    //logging: u128,
    //upstream_dns: u128,
}

pub struct ServerConfig {
    pub listeners: Vec<SocketConfig>
}

#[derive(Debug, Deserialize)]
pub struct SocketConfig {
    pub address: String,
    pub port: u16,
    pub protocol: Protocol,
}

pub enum Protocol {
    Udp,
    Tcp,
}

#[derive(Debug, Deserialize)]
pub struct TcpConfig {
    pub request_timeout_ms: Option<u64>,
    pub idle_timeout_ms: Option<u64>,
    pub max_connections: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RecursionConfig {
    pub enabled: bool,
    pub forwarders: Vec<String>,
}

pub struct StaticRecord {
    pub A: Option<Vec<String>>,
}

/// Represents the server socket configuration section in the config file.
#[derive(Deserialize)]
struct ServerSocket {
    address: String,
    port: u16,
}

enum Mode {
    Recursive,
    Authoritative,
    Cache,
    Forwarder,
}

fn parse_config(path: &str) -> std::io::Result<Config> {
    let file_contents = std::fs::read_to_string(path)?; // Read entire file contents into a string
    let yaml_config: FileConfig = serde_yaml::from_str(&file_contents)
        .map_err(|e: serde_yaml::Error| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let listen_addr: std::net::SocketAddr = format!("{}:{}", yaml_config.server_socket.address, yaml_config.server_socket.port)
        .parse()
        .map_err(|e: std::net::AddrParseError| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(
        Config {
        listen_addr,
        mode: Mode::Authoritative,
        forwarders: vec![],
    })
}

pub struct Config {
    pub listen_addr: SocketAddr,
    pub mode: Mode,
    pub forwarders: Vec<SocketAddr>,
}

impl Config {
    pub fn load_via_path (path: &str) -> std::io::Result<Self> {
        let config: Config = parse_config(path)?;
        Ok(config)
    }

    pub fn load_default () -> std::io::Result<Self> {
        let something: &str = "/etc/dns_server/config.toml";
        Self::load_via_path(something)
    }
}