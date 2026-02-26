mod dns_server;
mod config;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::env;
use std::error::Error;
use dns_server::DnsServer;
use config::Config;

/// Entry
fn main() {
    // TODO: If a configuration is provided by the user, use it. Otherwise use the default.
    let config_path = "config.yaml"; 
    let cnfg_result: Result<Config, Box<dyn Error>> = Config::load_via_path(config_path); 
    let cnfg: Config = match cnfg_result {
        Ok(c) => c,
        Err(e) => { 
            eprintln!("Failed to load configuration file. {}", e);
            return;
        }
    };

    // Create the DNS server using load configuration BEFORE running.
    let dns_server_result: Result<DnsServer, Box<dyn Error>> = DnsServer::new(cnfg);
    let dns_server = match dns_server_result { 
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create DNS server: {}", e);
            return; // Early break from main function here if dns_server_result returns an error
        }
    };
    
    // Start the DNS server
    if let Err(e) = dns_server.run() {
        eprintln!("Failed to start DNS server: {}", e); // If server fails to start, print error.
    }
    
    println!("DNS Server has stopped."); // This might not be nessesary

}