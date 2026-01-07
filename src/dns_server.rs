use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, sync::Arc, time::Instant};

pub struct CacheRecord {
    ip_addr: IpAddr,
    record_expiry: Instant // What is an Instant?

}

pub struct DnsCache {
    cache: HashMap<String, CacheRecord>,
}

/**
 * This struct is for a DNS Server
 * 
 */
pub struct DnsServer {
    // Network Variables
    socket_addr: SocketAddr,
    socket: UdpSocket,
    
    // Framework
    default_ipv4: Ipv4Addr,
    dns_map: HashMap<String, Ipv4Addr>,
    wildcard_enabled: bool,

    // Metric Data

    // Caching
    cache: Arc<DnsCache>,

    // Policy

}

/**
 * Implementation of the DNS Server struct
 */
impl DnsServer {
    pub fn new (socket_addr: SocketAddr, default_ipv4: Ipv4Addr) -> std::io::Result<Self> {
        let socket: UdpSocket = UdpSocket::bind(socket_addr)?;
        socket.set_nonblocking(false)?;

        // Returns an instance of the struct:
        Ok ( DnsServer {
            socket_addr: socket_addr,
            socket,
            default_ipv4,
            dns_map: HashMap::new(),
            wildcard_enabled: true,
            cache: Arc::new(DnsCache { cache: HashMap::new() })
            }
        )
    }

}
