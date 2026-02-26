use std::{array, net::{IpAddr, Ipv4Addr, SocketAddr}, path::PathBuf, str::FromStr};
use core::str;
use serde::Deserialize;
use std::error::Error;

type Result<T> = core::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct FileConfig {
    pub server: ServerConfig,
    //pub zones: Vec<ZoneConfig>
}

#[derive(Deserialize)]
struct ServerConfig {
    //mode: Option<Mode>,
    enable_tcp: bool,
    enable_https: bool,
    enable_tls: bool,
    basic_port: Option<u16>,
    tls_port: Option<u16>,
    https_port: Option<u16>,
    ip_addrs: Vec<IpAddr>,
}

/*
#[derive(Deserialize)]
struct ZoneConfig {
    namespace: String,
    zone_type: ZoneType,
    filepath: String
}

#[derive(Deserialize)]
enum ZoneType {
    Primary,
    Secondary,
    Stub
}

pub enum Protocol {
    UDP,
    TCP,
    HTTPS,
    TLS,
}

pub struct StaticRecord {
    pub A: Option<Vec<String>>,
}
*/
#[derive(Deserialize)]
enum Mode {
    Recursive,
    Authoritative,
    Cache,
    Forwarder,
}

/// DNS Config should have a list of valid internet socket addresses (either ipv4 or ipv6).
/// 
pub struct Config {
    pub mode: Mode,
    pub listeners: Vec<IpAddr>,
    pub enable_tcp: bool,
    pub enable_https: bool,
    pub enable_tls: bool, 
    pub std_listen_port: u16,
    pub https_listen_port: u16,
    pub tls_listen_port: u16,
    //pub forwarders: Vec<SocketAddr>, // Each domain has at least one authoritative DNS server that publishes information about that domain
}

impl Config {
    pub fn load_via_path (path: &str) -> Result<Config> {
        let config: Config = Self::parse_config(path)?;
        Ok(config)
    }

    pub fn load_default () -> Result<Config> {
        let something: &str = "/etc/dns_server/config.toml";
        Self::load_via_path(something)
    }

    pub fn parse_config (path: &str) -> Result<Config> {
        let contents = std::fs::read_to_string(path)?; // What happens if result is None for a `?`
        let yaml: FileConfig = serde_yaml::from_str(&contents)?; // Need `FileConfig` type explictly stated

        // Disassemble struct
        let FileConfig {
            server,
        } = yaml;

        // Custom or default ports
        let std_port = match server.basic_port {
            Some(thing) => thing,
            _ => 53
        };
        let tls_port = match server.tls_port {
            Some(thing) => thing,
            _ => 853
        };
        let http_port = match server.https_port {
            Some(thing) => thing,
            _ => 443
        };

        let mut addr_vec = Vec::new();
        for ip in server.ip_addrs {
            addr_vec.push(ip);
        }

        Ok( Config {
                mode: Mode::Authoritative,
                listeners: addr_vec,
                enable_tcp: server.enable_tcp,
                enable_https: server.enable_https,
                enable_tls: server.enable_tls,
                std_listen_port: std_port,
                https_listen_port: http_port,
                tls_listen_port: tls_port,
            }
        )   
    }
}