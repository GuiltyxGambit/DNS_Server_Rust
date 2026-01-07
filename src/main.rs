mod dns_server;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::env;

use trust_dns_server::proto::rr::rdata::null;
use dns_server::DnsServer;



//mod dns;
//use dns::DNS_Server;


fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Validate CLI args:
    if args.len() == 2 {
        eprintln!("Usage: {} <search_term> <file_path>", args[0]);
    }

    let search_term: &String = &args[1];

    
    let address: &str = "0.0.0.0:8080";

    let socket: UdpSocket = UdpSocket::bind(address)?;

    let ip_address = "1.1.1.1";
    let mac_address = "AA:BB:CC:BBC";

    let m = DnsServer::new(ip_address, mac_address);

    //let m = DNS_Server::new("Test");
    //println!("This is a {}", m.word);

    println!("Hello, world!");
    Ok(println!("End"))
}