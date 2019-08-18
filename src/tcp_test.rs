use tokio::net::TcpStream;
use tokio::prelude::*;
use std::net::ToSocketAddrs;
use tokio::io::{read_to_end, write_all};
use std::fs::File;
use std::net::SocketAddr;

const REQ_BODY: &[u8] = b"GET /json HTTP/1.1\r\nHost: httpbin.org\r\nConnection: close\r\n\r\n";

fn resolve_addr(host: &str) -> Result<SocketAddr, String> {
    let mut addr_iter = match host.to_socket_addrs() {
        Ok(addr_iter) => addr_iter,
        Err(e) => return Err(format!("Invalid host name {:?}: {:?}", host, e)),
    };
    match addr_iter.next() {
        None => Err(format!("No addresses found for host: {:?}", host)),
        Some(addr) => Ok(addr),
    }
}


fn download(host: String, path: String, filename: String) -> impl Future<Item=(), Error=()> {
    let addr = match resolve_addr(&host) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Error resolving address: {}", e);
            return future::Either::A(future::err(()));
        }
    };
    let req_body = format!(
        "GET {} HTTP/1.1\r\nHost: {}:80\r\nConnection: close\r\n\r\n",
        path,
        host,
    );

    future::Either::B(TcpStream::connect(&addr)
        .and_then(|stream| {
            write_all(stream, req_body).and_then(|(stream, _body)| {
                let buffer = vec![];
                read_to_end(stream, buffer).and_then(|(_stream, buffer)| {
                    File::create(filename).and_then(|mut file| {
                        file.write_all(&buffer)
                    })
                })
            })
        }).map_err(|e| eprintln!("Error occured: {:?}", e)))
}

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
    tokio::run(future);

    let future = TcpStream::connect(&addr)
        .and_then(|stream| {
            write_all(stream, REQ_BODY)
        }).and_then(|(stream, _body)| {
        let buffer = vec![];
        read_to_end(stream, buffer)
    }).map(|(_stream, buffer)| {
        let s = std::str::from_utf8(&buffer).unwrap();
        println!("{}", s);
    }).map_err(|e| eprintln!("Error occured: {:?}", e));
    tokio::run(future)
}

fn tcp_spawn_cli() {
    tokio::run(future::poll_fn(|| {
        let mut args = std::env::args().skip(4);
        loop {
            match (args.next(), args.next(), args.next()) {
                (Some(host), Some(path), Some(filename)) => {
                    tokio::spawn(download(host, path, filename));
                }
                _ => return Ok(Async::Ready(())),
            }
        }
    }))
}
#[test]
fn tcp_client_test() {
    tcp_test();
}

#[test]
fn tcp_spawn_cli_test() {
    tcp_spawn_cli();
}