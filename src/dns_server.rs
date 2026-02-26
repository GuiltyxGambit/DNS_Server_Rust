use std::{any::Any, collections::HashMap, fmt::Result, net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket}, sync::Arc, time::Instant, vec};
use trust_dns_server::proto::op::query;
use crate::config::Config;

type Result<T> = core::result::Result<T, Box<dyn Error>>;

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

pub struct DnsServer {
    udp_listeners: Vec<UdpSocket>,
    tcp_listeners: Vec<TcpListener>,
    tls_listeners: Vec<TcpListener>,   // TLS wraps TCP
    https_listeners: Vec<TcpListener>, // HTTP also starts as TCP
    dns_map: HashMap<String, Ipv4Addr>,
}

/// Implementation of DnsServer
impl DnsServer {

    /// Initialize the different sockets based on the configuration
    pub fn new (config: Config) -> Result<DnsServer> {
        println!("Initializing DNS Server with Config.");
        let Config {
            mode,
            listeners,
            enable_tcp,
            enable_https,
            enable_tls,
            std_listen_port,
            https_listen_port,
            tls_listen_port,
        } = config; 

        let mut udp_vector = Vec::<UdpSocket>::new();
        let mut tcp_vector = Vec::<TcpListener>::new();
        let mut https_vector = Vec::new();
        let mut tls_vector = Vec::new();

        for listener in listeners {
            let addr = SocketAddr::new(listener, std_listen_port);
            let udp_socket = UdpSocket::bind(addr);
            if udp_socket.is_ok() {
                udp_vector.push(udp_socket.unwrap());
            } else {
                println!("The UDP socket {addr} : {std_listen_port} failed to bind.");
            }
        }

        if enable_tcp {
            for listener in listeners {
                let tcp_addr = SocketAddr::new(listener, std_listen_port);
                match TcpListener::bind(tcp_addr) {
                    Ok(listener) => {
                        println!("Binding {}");
                        tcp_vector.push(listener);
                    },
                    Err(e) => {
                        eprintln!("Failed to bind TCP {}: {}", tcp_addr, e);
                    },
                }
            }
        }

        // Create TLS Sockets: (DO LATER)

        // Create HTTPS Sockets (DO LATER)

        Ok (DnsServer {
            udp_listeners: udp_vector,
            tcp_listeners: tcp_vector,
            https_listeners: https_vector,
            tls_listeners: tls_vector,
            dns_map: HashMap::new(),
            }
        )
    }

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

    /// This function should only run when a packet has been recieved. Can use for testing?
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

    fn handle (&self) {
        let mut buffer: [u8; 512] = [0u8; 512]; // DNS packets are max 512 bytes (UDP)

        loop { 
            let (size, src_addr) = match self.socket.recv_from(&mut buffer) {
                Ok((size, src_addr)) => (size, src_addr),
                Err(e) => {
                    eprintln!("Failed to receive data: {}", e);
                    continue;
                }
            };
            // Add a packet data validator here??
            println!("Packet size: {}", size);

            let request_data= &buffer[..size];

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

    /// Run the DNS server
    /// 
    /// How can I accomodate for multiple protocols?
    pub fn run (&self) -> std::io::Result<()> {
        println!("DNS Server is running.");
        

        Ok(())
    }
}
