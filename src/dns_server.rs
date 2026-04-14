// External Crates
use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::net::{UdpSocket as StdUdpSocket, TcpListener as StdTcpListener};
use std::str::FromStr;
use std::time::Instant;
use tokio::net::{TcpListener, UdpSocket};
use tokio::task::JoinSet;
use tokio_util::bytes::{self, Buf, Bytes, BytesMut};
use tokio_util::udp::UdpFramed;
use tokio_util::codec::BytesCodec;
use futures::StreamExt;

// Internal Crates
use crate::config::Config;

// Temporary Error type
type Result<T> = core::result::Result<T, Box<dyn Error + Send + Sync>>;

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
pub struct ResourceRecord {
    pub name:    String,
    pub rtype:   u16,
    pub rclass:  u16,
    pub ttl:     u32,
    pub rdata:   Vec<u8>, // For A records: 4 bytes of IPv4
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
    udp_listeners: Vec<StdUdpSocket>,
    tcp_listeners: Vec<StdTcpListener>,
    tls_listeners: Vec<TcpListener>,   // TLS wraps TCP
    https_listeners: Vec<TcpListener>, // HTTP also starts as TCP
    dns_map: HashMap<String, Ipv4Addr>,
}

/// Implementation of DnsServer
impl DnsServer {

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

        let mut udp_vector = Vec::<StdUdpSocket>::new();
        let mut tcp_vector = Vec::<StdTcpListener>::new();
        let mut https_vector = Vec::new();
        let mut tls_vector = Vec::new();

            // Need to borrow `listeners` in order to reuse it in subsequent checks
            // Iterating over reference to vector `listeners`.
            for listener in &listeners { // Borrow listeners

                let addr = SocketAddr::new(*listener, std_listen_port);

                let udp_socket = StdUdpSocket::bind(addr);
                match udp_socket {
                    Ok(socket) => {
                        udp_vector.push(socket);
                    }
                    Err(e) => {
                        eprintln!("Failed to bind std UDP {}: {}", addr, e);
                    }
                }

                if enable_tcp {
                    match StdTcpListener::bind(addr) {
                        Ok(listener) => {
                            tcp_vector.push(listener);
                        }
                        Err(e) => {
                            eprintln!("Failed to bind TCP {}: {}", addr, e);
                        }
                    }
                }

                // TODO: implement TLS
                if enable_tls {
                    let tls_socket = SocketAddr::new(*listener, tls_listen_port);
                }

                // TODO: implement HTTPS
                if enable_https {
                    let https_socket = SocketAddr::new(*listener, https_listen_port);
                }

            }

        // https_listeners and tls_listeners currently have empty vectors
        Ok (DnsServer {
            udp_listeners: udp_vector,
            tcp_listeners: tcp_vector,
            https_listeners: https_vector,
            tls_listeners: tls_vector,
            dns_map: HashMap::new(), // Replace later.
            }
        )
    }

    /// This function is designed to give back a simple instance of a DNS server for testing. No complications of invalid sockets. 
    pub fn new_test(cfg: Config) -> Result<DnsServer> {
        
        // Find a valid UDP socket for system and add it to the UDP vector. 
        let ip = IpAddr::from_str("0.0.0.0")?;
        let port = 0505;
        let addr = SocketAddr::new(ip, port);
        let socket = StdUdpSocket::bind(addr)?;

        let mut udp_listeners = vec![socket];

        Ok(DnsServer { 
            udp_listeners, 
            tcp_listeners: Vec::<StdTcpListener>::new(), 
            tls_listeners: Vec::new(), 
            https_listeners: Vec::new(), 
            dns_map: HashMap::new() 
        })
    }

    
    pub async fn udp_listener<T: UdpHandler>(socket: UdpSocket, udp_handler: T) -> std::io::Result<()> { // Using `std::io::Result<()>` because dynamic errors cannot be sent between threads safely. 
        println!("UDP Socket listening");

        // `framed` is a UdpFramed a stream interface defined for a BytesCodec.
        let mut framed: UdpFramed<BytesCodec> = UdpFramed::new(socket, BytesCodec::new());

        // 
        while let Some(result) = framed.next().await {
            let (bytes, peer_addr) = result?;
            udp_handler.handle(bytes.freeze()).await?; // freeze() = BytesMut -> Bytes (owned, Arc-backed)
        }

        Ok(())
    }

    fn parse_qname(mut slice: &[u8]) -> Result<(String, &[u8])> {
        let mut labels: Vec<String> = Vec::new();

        if slice.is_empty() {
            return Err("Unexpected end of QNAME".into());
        }

        // Fix later
        while (slice[0] != 0) {
            let len = slice[0] as usize;
            let label = std::str::from_utf8(&slice[..len])?;
            labels.push(label.to_owned());
            slice = &slice[len..];
        }
        Ok((labels.join("."), slice))
    }
    
    pub fn parse_questions(num_questions: u16, mut slice: &[u8]) -> Result<(Vec<DNSQuery>, &[u8])> {
        let mut questions = Vec::with_capacity(num_questions as usize);
        for i in 0..num_questions {
            let (query, temp_slice) = Self::parse_question(slice)?;
            questions.push(query);
            slice = temp_slice;
        }
        Ok((questions, slice))
    }

    pub fn parse_question(slice: &[u8]) -> Result<(DNSQuery, &[u8])> {
        // Read specified domain name.
        let (name, rest) = Self::parse_qname(slice)?;

        if rest.len() < 4 {
            return Err("Not enough bytes for QTYPE/QCLASS".into());
        }

        Ok((
            DNSQuery {
                name,
                qtype:  u16::from_be_bytes([rest[0], rest[1]]),
                qclass: u16::from_be_bytes([rest[2], rest[3]]),
            },
            &rest[4..],
        ))
    }

    pub fn parse_header(bytes: &[u8]) -> Result<(DNSHeader, &[u8])> {
        if bytes.len() < 12 {
            return Err("Packet too short for DNS header".into());
        }
        
        // Directly index:
        let header = DNSHeader {
            id: u16::from_be_bytes([bytes[0],  bytes[1]]),
            flags: u16::from_be_bytes([bytes[2],  bytes[3]]),
            qdcount: u16::from_be_bytes([bytes[4],  bytes[5]]),
            ancount:  u16::from_be_bytes([bytes[6],  bytes[7]]),
            nscount: u16::from_be_bytes([bytes[8],  bytes[9]]),
            arcount:  u16::from_be_bytes([bytes[10],  bytes[11]]),
        };

        let (left, right) = bytes.split_at(12);

        Ok((header, right))
    }

    pub fn resolve_questions(query_vector: Vec<DNSQuery>) -> Result<Vec<ResourceRecord>> {
        let mut answers = Vec::new();
        
        /**
        for question in query_vector {
            // Resolve single domain-name query
            if let Some(ip) = self.dns_map.get(&question.name) {
                answers.push(ResourceRecord {
                    name:   question.name.clone(),
                    rtype:  1,              // A
                    rclass: 1,              // IN
                    ttl:    300,
                    rdata:  ip.octets().to_vec(),
                });
            }
        }
        */

        Ok(answers)
    }

    // Build responce should be the same regardless of protocol. 
    pub fn build_response(header: &DNSHeader, vector: &Vec<ResourceRecord>) -> Result<Bytes> {

        todo!()
    }

    /// This function takes ownership of `self`, (instance of DNS server).
    pub async fn run (mut self) -> std::io::Result<()> { // Returns a `Future`, 
        println!("DNS Server is running.");

        let mut tasks = JoinSet::new(); 

        for std_socket in self.udp_listeners.drain(..) { // drain() moves each socket out of the Vec. 
            std_socket.set_nonblocking(true)?; // Async IO requires non-blocking sockets so that polling a socket never stalls a thread
            let tokio_socket = UdpSocket::from_std(std_socket)?;

            // Using a joinset with a handle udp should be adequate.
            tasks.spawn(async move {
                Self::udp_listener(tokio_socket, 
                    // udp_handler:
                    |bytes: Bytes| async move {                        
                        // Parse DNS Header (12 bytes):

                        // Note: Using `self` in this creates FnOnce issues. Need to investigate solution later.
                        let (header, message) = Self::parse_header(&bytes)
                            .map_err(std::io::Error::other)?;

                        if (header.flags & (1 << 15 )) != 0 { // DNS message is a response
                            return Ok(None);
                        }
                            
                        let (query_vector, remaining_slice) = Self::parse_questions(header.qdcount, message)
                            .map_err(std::io::Error::other)?;

                        let answers = Self::resolve_questions(query_vector)
                            .map_err(std::io::Error::other)?;

                        let response_bytes = Self::build_response(&header, &answers)
                            .map_err(std::io::Error::other)?;

                        let response_bytes = Bytes::new();

                        Ok(Some(response_bytes))
                    }
                ).await // Ownership of socket is given to the loop.
            });
        }

        let signal = self.block_until_done(tasks).await;
        Ok(())
    }

    pub async fn block_until_done(&mut self,  mut tasks: JoinSet<std::io::Result<()>>) -> std::io::Result<()> {
        // Loop exits if `tasks.join_next().await` returns None. Otherwise, `Some` should contain a Result.
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok(())) => {
                    eprintln!("Warning: a listener task exited unexpectedly.");
                }
                Ok(Err(e)) => {
                    // A listener loop returned an error, thus shut everything.
                    eprintln!("Listener task failed: {}", e);
                    tasks.abort_all();
                    return Err(e);
                }
                Err(join_err) => {
                    // A task paniced or was cancelled.
                    eprintln!("Task join error: {}", join_err);
                    tasks.abort_all();
                    return Err(std::io::Error::other(join_err));
                }
            }
        }
        Ok(())
    }


}

struct UDPHandler {
    socket: UdpSocket
}

impl UDPHandler {
    pub fn new() -> () {
        todo!()
    }

    pub fn parse_header(bytes: &[u8]) -> Result<(DNSHeader, &[u8])> {
        if bytes.len() < 12 {
            return Err("Packet too short for DNS header".into());
        }
        
        // Directly index:
        let header = DNSHeader {
            id: u16::from_be_bytes([bytes[0],  bytes[1]]),
            flags: u16::from_be_bytes([bytes[2],  bytes[3]]),
            qdcount: u16::from_be_bytes([bytes[4],  bytes[5]]),
            ancount:  u16::from_be_bytes([bytes[6],  bytes[7]]),
            nscount: u16::from_be_bytes([bytes[8],  bytes[9]]),
            arcount:  u16::from_be_bytes([bytes[10],  bytes[11]]),
        };

        let (left, right) = bytes.split_at(12);

        Ok((header, right))
    }

    pub fn parse_qname(mut slice: &[u8]) -> Result<(String, &[u8])> {
                let mut labels: Vec<String> = Vec::new();

        if slice.is_empty() {
            return Err("Unexpected end of QNAME".into());
        }

        // Fix later
        while (slice[0] != 0) {
            let len = slice[0] as usize;
            let label = std::str::from_utf8(&slice[..len])?;
            labels.push(label.to_owned());
            slice = &slice[len..];
        }
        Ok((labels.join("."), slice))
    }

    pub fn parse_question(slice: &[u8]) -> Result<(DNSQuery, &[u8])> {
        // Read specified domain name.
        let (name, rest) = Self::parse_qname(slice)?;

        if rest.len() < 4 {
            return Err("Not enough bytes for QTYPE/QCLASS".into());
        }

        Ok((
            DNSQuery {
                name,
                qtype:  u16::from_be_bytes([rest[0], rest[1]]),
                qclass: u16::from_be_bytes([rest[2], rest[3]]),
            },
            &rest[4..],
        ))
    }

    pub fn parse_questions(num_questions: u16, mut slice: &[u8]) -> Result<(Vec<DNSQuery>, &[u8])> {
        let mut questions = Vec::with_capacity(num_questions as usize);
        for i in 0..num_questions {
            let (query, temp_slice) = Self::parse_question(slice)?;
            questions.push(query);
            slice = temp_slice;
        }
        Ok((questions, slice))
    }

    pub fn resolve() {

    }

    pub fn build_response() -> () {

    }
}

impl UdpHandler for UDPHandler {
    async fn handle(&self, input: Bytes) -> std::io::Result<Option<Bytes>> {

        let bytes = input; // really corny temporary fix

        let (header, message) = Self::parse_header(&bytes)
            .map_err(std::io::Error::other)?;

        // Ignore responses
        if (header.flags & (1 << 15)) != 0 {
            return Ok(None);
        }

        let (query_vector, remaining_slice) = Self::parse_questions(header.qdcount, message)
            .map_err(std::io::Error::other)?;

        todo!()
    }
}

trait UdpHandler {
    async fn handle(&self, input: Bytes) -> std::io::Result<Option<Bytes>>;
}

impl<F, Fut> UdpHandler for F
where
    F: Fn(Bytes) -> Fut,
    Fut: Future<Output = std::io::Result<Option<Bytes>>>,
{
    async fn handle(&self, input: Bytes) -> std::io::Result<Option<Bytes>> {
        self(input).await
    }
}

// A protocol handler consists of a `request handler` and a `response handler`?
