use tokio::net::TcpStream;
use tokio::prelude::*;
use std::net::ToSocketAddrs;

fn tcp_test() {
    let mut addr_iter = "httpbin.org:80".to_socket_addrs().unwrap();
    let addr = match addr_iter.next() {
        None => panic!("DNS resolution failed"),
        Some(addr) => addr,
    };
    let future = TcpStream::connect(&addr)
        .map_err(|e| eprintln!("Error connecting: {:?}", e))
        .map(|stream| {
            println!("Got a stream: {:?}", stream);
        });
    tokio::run(future)
}

#[test]
fn tcp_client_test() {
    tcp_test();
}