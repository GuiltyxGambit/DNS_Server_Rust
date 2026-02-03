mod dns_server;
mod config;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::env;
use dns_server::DnsServer;
use config::Config;

/// Bootstrapper
fn main() {
    let _args: Vec<String> = env::args().collect(); // Configure this later

    // TODO: If a configuration is provided by the user, use it. Otherwise use the default.
    let config_path = "C:/Users/bwroc/Documents/Projects/DNS_Server_Rust/src/config.yaml";
    let cnfg_result: Result<Config, std::io::Error> = config::Config::load_via_path(config_path);
    let cnfg: Config = match cnfg_result {
        Ok(c) => c,
        Err(e) => { 
            eprintln!("!!! FAILED TO LOAD CONFIG. ERROR MESSAGE: {}", e);
            return;
        }
    };

    // Create the DNS server using load configuration BEFORE running.
    let dns_server_result: Result<DnsServer, std::io::Error> = DnsServer::new(cnfg);
    let dns_server: DnsServer = match dns_server_result {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create DNS server: {}", e);
            return;
        }
    };
    
    // Start the DNS server
    if let Err(e) = dns_server.run() {
        eprintln!("Failed to start DNS server: {}", e); // If server fails to start, print this error.
    }
    
    println!("DNS Server has stopped.");

}