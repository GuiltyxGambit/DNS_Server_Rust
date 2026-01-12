mod dns_server;
mod config;
mod config_parser;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::env;
use dns_server::DnsServer;
use config::Config;


fn main() {
    //let args: Vec<String> = env::args().collect(); // Configure this later
    let cfg_result: Result<Config, std::io::Error> = config::Config::load_via_path("C:/Users/bwroc/Documents/Projects/DNS_Server_Rust/src/config.yaml");
    let cfg: Config = match cfg_result {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    let dns_server_result: Result<DnsServer, std::io::Error> = DnsServer::new(cfg);
    let dns_server: DnsServer = match dns_server_result {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create DNS server: {}", e);
            return;
        }
    };

    //let m = DNS_Server::new("Test");
    //println!("This is a {}", m.word);
}