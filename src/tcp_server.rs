use tokio::prelude::*;
use tokio::net::TcpListener;
use tokio::io::{copy, write_all};

fn tcp_server() {
    let addr = "127.0.0.1:3000".parse().expect("couldn't parse address");
    let listener = TcpListener::bind(&addr).expect("couldn't bind address");
    let future = listener
        .incoming()
        .for_each(|socket| {
            let (reader, writer) = socket.split();
            write_all(writer, b"Welcome to the echo server\r\n")
                .and_then(|(writer, _)| {
                    copy(reader, writer)
                        .map(|_| println!("Connection closed"))
                })
        })
        .map_err(|e| eprintln!("An error occurred: {:?}", e))
        ;
    tokio::run(future)
}

#[test]
fn tcp_server_test() {
    tcp_server();
}