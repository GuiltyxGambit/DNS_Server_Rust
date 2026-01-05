use std::net::{SocketAddrV4, SocketAddr};
use trust_dns_server::server::{Request, RequestHandler, ResponseHandler, ServerFuture};
use trust_dns_server::proto::op::ResponseCode;
use trust_dns_server::proto::rr::{DNSClass, Name, RData, Record, RecordType};
use trust_dns_server::proto::rr::rdata::A;

mod dns;
use dns::DNS_Server;

struct SimpleHandler;

fn main() {
    let handler: SimpleHandler; 
    handler = SimpleHandler; // Assign a new instance of SimpleHandler. Does not need "new" keyword.

    let m = DNS_Server::new("Test");
    println!("This is a {}", m.word)
    //println!("Hello, world!");
}
