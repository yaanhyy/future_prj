use tokio::net::{TcpStream, tcp::ConnectFuture};
use futures::{Future, Async, Poll};
use std::time::Duration;
use std::thread::{sleep};
use tokio::io::AsyncWrite;
use bytes::{Bytes, Buf};
use std::io::{self, Cursor};
use tokio::prelude::*;


use futures;
struct GetPeerAddr {
    connect: ConnectFuture,
}

impl Future for GetPeerAddr {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.connect.poll() {
            Ok(Async::Ready(socket)) => {
                println!("peer address = {}", socket.peer_addr().unwrap());
                Ok(Async::Ready(()))
            }
            Ok(Async::NotReady) => {
                println!("not ready");
                Ok(Async::NotReady)
            },
            Err(e) => {
                println!("failed to connect: {}", e);
                Ok(Async::Ready(()))
            }
        }
    }
}



// HelloWorld 有两个状态, 即等待连接的状态和已经连接的状态
enum HelloWorld {
    Connecting(ConnectFuture),
    Connected(TcpStream, Cursor<Bytes>),
}

impl Future for HelloWorld {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        use self::HelloWorld::*;

        loop {
            match self {
                Connecting(ref mut f) => {
                    let socket = try_ready!(f.poll());
                    let data = Cursor::new(Bytes::from_static(b"hello world"));
                    *self = Connected(socket, data);
                }
                Connected(ref mut socket, ref mut data) => {
                    // Keep trying to write the buffer to the socket as long as the
                    // buffer has more bytes available for consumption
                    while data.has_remaining() {
                        try_ready!(socket.write_buf(data));
                    }
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}


fn async_future() {
    let addr = "127.0.0.1:1234".parse().unwrap();
    let connect_future = TcpStream::connect(&addr);
    let get_peer_addr = GetPeerAddr {
        connect: connect_future,
    };
    let duration = Duration::from_millis(1000);
    sleep(duration);
    tokio::run(get_peer_addr);
}

fn async_chain() {
    let addr = "127.0.0.1:1234".parse().unwrap();
    let connect_future = TcpStream::connect(&addr);
    let hello_world = HelloWorld::Connecting(connect_future);

    // 运行之
    tokio::run(hello_world.map_err(|e| println!("{0}", e)))
}

#[test]
fn async_test() {
    async_future();
}

#[test]
fn async_chain_test() {
    async_chain();
}