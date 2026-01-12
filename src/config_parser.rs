use core::str;

use crate::config::{Config, Mode};
use serde::Deserialize;
use trust_dns_server::store::file;

 /// ConfigParser struct is just a namespace for `parse_file` function
pub struct ConfigParser {
    //pub config: Config,
}

/// Internal structs to help with deserialization. `FileConfig` represents the overall structure of the config file.
#[derive(Deserialize)]
struct FileConfig {
    socket: ServerSocket,
    //logging: u128,
    //upstream_dns: u128,
}

/// Represents the server socket configuration section in the config file.
#[derive(Deserialize)]
struct ServerSocket {
    address: String,
    port: u16,
}

/// What the hell is going on here?
impl ConfigParser {
    pub fn parse_file(path: &str) -> std::io::Result<Config> { 
        let file_content: String = std::fs::read_to_string(path)?; 
        // Deserialize the YAML content into the FileConfig struct. FileConfig is inferred. 
        let file_config: FileConfig = serde_yaml::from_str(&file_content)
            // Converts serde_yaml::Error to std::io::Error
            .map_err(|e: serde_yaml::Error| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
        // At this point, the file_config contains a structured representation of the config file.

        // 
        let listen_addr: std::net::SocketAddr = format!("{}:{}", file_config.socket.address, file_config.socket.port)
            // Parse on what? Line above gives String, which implements `from_str` trait. 
            // `.parse()` translates is replaced by the  `from_str` method of `SocketAddr`? 
            .parse()
            // Converts AddrParseError to std::io::Error. Why not keep as AddrParseError?
            .map_err(|e: std::net::AddrParseError| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(
            Config {
            listen_addr,
            mode: Mode::Authoritative,
            forwarders: vec![],
        })
    }
}