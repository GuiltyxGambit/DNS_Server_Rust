use std::{array, net::{IpAddr, Ipv4Addr, SocketAddr}, path::PathBuf, str::FromStr};
use core::str;
use serde::Deserialize;
use std::error::Error;

// What is this doing...?
type Result<T> = core::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct FileConfig {
    pub server: ServerConfig,
    //pub zones: Vec<ZoneConfig>
}

#[derive(Deserialize)]
struct ServerConfig {
    //mode: Option<Mode>,
    disable_tcp: bool,
    disable_https: bool,
    disable_tls: bool,
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


pub struct Config {
    pub listen_addr: SocketAddr,
    pub mode: Mode,
    pub forwarders: Vec<SocketAddr>, // Each domain has at least one authoritative DNS server that publishes information about that domain
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

    pub fn parse_config (path: &str) -> Result<Config>{
        let contents = std::fs::read_to_string(path)?; // What happens if result is None for a `?`
        let yaml: FileConfig = serde_yaml::from_str(&contents)?;

        // Decompose struct
        let FileConfig {
            server,
        } = yaml;

        let port = match server.basic_port {
            Some(thing) => thing,
            _ => 53
        };

        Ok (Config {
            listen_addr: SocketAddr::new(server.ip_addrs[0],port),
            mode: Mode::Authoritative,
            forwarders: vec![],
            }
        )   
    }
}