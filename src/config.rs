use std::{array, net::SocketAddr, path::PathBuf};
use crate::config_parser::ConfigParser;


pub enum Mode {
    Authoritative,
    Forwarder,
}

pub struct Config {
    pub listen_addr: SocketAddr,
    pub mode: Mode,
    pub forwarders: Vec<SocketAddr>,
}

impl Config {
    pub fn load_via_path (path: &str) -> std::io::Result<Self> {
        let config: Config = ConfigParser::parse_file(path)?;
        Ok(config)
    }

    pub fn load_default () -> std::io::Result<Self> {
        //let something: &str = self::default_path();
        let something: &str = "/etc/dns_server/config.toml";
        Self::load_via_path(something)
    }

}