use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, sync::Arc, time::Instant, vec};
use trust_dns_server::proto::op::query;

use crate::config::Config;

/// Add later
pub struct CacheRecord {
    ip_addr: IpAddr,
    record_expiry: Instant,
}

/// Add later
pub struct DnsCache {
    cache: HashMap<String, CacheRecord>,
}

#[derive(Debug)]
struct DNSQuery {
    name: String,
    qtype: u16,
    qclass: u16,
}

#[derive(Debug)]
struct DNSHeader {
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

/// Generic handler trait for different types of requests
trait Handler<T> {
    type Raw;
    fn handle(&self, item: T);
}

/// Main DNS Server struct
pub struct DnsServer {
    // Network Variables
    socket_addr: SocketAddr,
    socket: UdpSocket,
    
    // Framework
    default_ipv4: Ipv4Addr,
    dns_map: HashMap<String, Ipv4Addr>,
    wildcard_enabled: bool,

    // Metric Data (Add later) 

    // Caching
    cache: Arc<DnsCache>,

    // Policy (Add later)

}

/// Implementation block for DnsServer
impl DnsServer {
    pub fn new (config: Config) -> std::io::Result<Self> {
        let socket_addr: SocketAddr = config.listen_addr;
        let default_ipv4: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0); // Placeholder for default IP
        let socket: UdpSocket = UdpSocket::bind(socket_addr)?;
        socket.set_nonblocking(false)?;
        Ok (DnsServer {
            socket_addr: socket_addr,
            socket,
            default_ipv4,
            dns_map: HashMap::new(),
            wildcard_enabled: true,
            cache: Arc::new(DnsCache { cache: HashMap::new() })
            }
        )
    }

    /// QNames in DNS are length prefixed string-labels
    /// A QName string is terminated by [0]
    /// Example: [3] w w w [6] g o o g l e [3] c o m [0]
    fn parse_qname (&self, pkt: &[u8], mut offset: usize) -> std::io::Result<(String, usize)> {
        let mut labels = Vec::new();
        loop { 
            if pkt.len() < offset {
                return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData, // Not the right error, but good enough
                "Unexpected end of buffer while parsing QNAME",
            ));
            }
            let len: usize = pkt[offset] as usize; // Read length prefix
            offset += 1;
            if len == 0 { 
                // Break loop of reading labels iff the terminating character is read
                break; 
            }
            else if offset + len > pkt.len() {
                // Return a data error if the length of the label to be read exceeds the buffer
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Label length exceeds buffer",
                ));
            }
            else {
                // Read the label based on the length prefix
                let label = std::str::from_utf8(&pkt[offset..offset + len])
                .map_err(|_| std::io::Error::new( // What is the point of mapping an error?
                    std::io::ErrorKind::InvalidData,
                    "Invalid UTF-8 in QNAME",
                ))?;
                // Put the label into the string vector
                labels.push(label.to_string());
                offset += len; // Increment offset by length of the string.
            }
        }
        Ok((labels.join("."), offset)) // Eg www.google.com
    }

    fn parse_dns_query (&self, pkt: &[u8], offset: usize) -> std::io::Result<(DNSQuery, usize)> {
        // First get variable length 
        let (name, offset) = self.parse_qname(pkt, offset)?;
        // Check if there is even a question
        if offset + 4 > pkt.len() { 
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "DNS QUESTION AND/OR TYPE & CLASS TRUNCATED FROM THE PACKET",
            ));
        }
        let qtype = u16::from_be_bytes([pkt[offset], pkt[offset + 1]]);
        let qclass = u16::from_be_bytes([pkt[offset + 2], pkt[offset + 3]]);
        Ok(( // Return tuple of DNSQuestion and new offset value
            DNSQuery {name, qtype, qclass}, 
            offset + 4
        ))
    } 

    fn parse_dns_queries (&self, pkt: &[u8], offset: usize, num: usize) -> std::io::Result<(Vec<DNSQuery>, usize)> {
        // Need to know number of queries to determine how many times to loop.
        let mut vec_queries = Vec::new();
        for i in 0..num {
            let (query,_) = self.parse_dns_query(pkt, offset)?;
            vec_queries.push(query);
        }
        Ok((vec_queries, offset))
    }

    fn resolve_hostnames () -> () {

    }

    /// 
    fn handle_request(&self, pkt: &[u8]) -> std::io::Result<Vec<u8>> {
        // Parse DNS header
        let len = pkt.len();
        if len < 12 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Packet too short"));
        }

        let mut offset: usize = 0;
        let header = DNSHeader {
            id: u16::from_be_bytes([pkt[0], pkt[1]]),
            flags: u16::from_be_bytes([pkt[2], pkt[3]]),
            qdcount: u16::from_be_bytes([pkt[4], pkt[5]]),
            ancount: u16::from_be_bytes([pkt[6], pkt[7]]),
            nscount: u16::from_be_bytes([pkt[8],pkt[9]]),
            arcount: u16::from_be_bytes([pkt[10],pkt[11]]),
        };

        offset = 12; // Offset after DNS header should always be 12 bytes

        // 2. Parse DNS questions
        let query_vector = self.parse_dns_queries(pkt, offset,header.qdcount as usize);

        // 3. Extract hostname

        // 4. Resolve hostname to IPv4

        // 5. Build DNS response packet

        // Placeholder:
        Ok(vec![])
    }

    /// Run the DNS server
    /// Note how this is a very low level way of doing things compared to Java. There is no buffered reader/writer abstraction.
    pub fn run (&self) -> std::io::Result<()> {
        println!("DNS Server is running on {}", self.socket_addr);

        let mut buffer: [u8; 512] = [0u8; 512]; // DNS packets are max 512 bytes (UDP)

        loop {
            // Receive data from clients
            let (size, src_addr) = match self.socket.recv_from(&mut buffer) { // writes to the buffer
                Ok((size, src_addr)) => (size, src_addr),
                Err(e) => {
                    eprintln!("Failed to receive data: {}", e);
                    continue;
                }
            };

            let request_data: &[u8] = &buffer[..size]; // Slice the buffer to the actual size received

            // Handle the DNS request
            let response = match self.handle_request(request_data) {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Failed to handle request: {}", e);
                    continue;
                }
            };

            // Send response back to the client
             if let Err(e) = self.socket.send_to(&response, src_addr) {
                eprintln!("send_to failed: {}", e);
            };

        }
    }
}
