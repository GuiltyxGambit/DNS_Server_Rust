mod dns_server;
mod config;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::env;
use std::error::Error;
use dns_server::DnsServer;
use config::Config;
use tokio::runtime::{Runtime};

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

    // let dns_server_result = DnsServer::new(cnfg);

    // Current stage of build needs a DNS Server with a basic port open. `new_test()` gives an instance of struct 
    // that has a simple predetermined valid UDP socket. 
    let dns_server_result = DnsServer::new_test(cnfg);

    let dns_server: DnsServer = match dns_server_result { 
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create DNS server: {}", e);
            return; // Early break from main function here if dns_server_result returns an error
        }
    };
    
    // Create a default multi-threaded runtime. 
    // This Tokio runtime has the same number of threads as CPU cores.
    // 
    let runtime = Runtime::new().unwrap();

    
    // A future is given to runtime.block_on(...) with the expectation the argument spawning tasks 
    let future = dns_server.run();

    // Start the DNS server using Tokio. OS thread is blocked here until runtime conditions conclude.
    if let Err(e) = runtime.block_on(future) {
        eprintln!("Failed to start DNS server: {}", e);
    }
    
    println!("DNS Server has stopped."); // This might not be nessesary

}