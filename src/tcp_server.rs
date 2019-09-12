use tokio::prelude::*;
use tokio::net::TcpListener;
use tokio::io::{copy, write_all};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_timer::Delay;
use tokio_tcp::{ConnectFuture, Incoming, TcpStream};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    iter::{self, FromIterator},
    net::{IpAddr, SocketAddr},
    time::{Duration, Instant},
    vec::IntoIter
};
use bytes::{BufMut, Bytes, BytesMut};


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


/// Incoming connection stream which pauses after errors.
#[derive(Debug)]
struct Listener {
    /// The incoming connections.
    stream: Incoming,
    /// The current pause if any.
    pause: Option<Delay>,
    /// How long to pause after an error.
    pause_duration: Duration
}

impl Listener {
    fn new(stream: Incoming, duration: Duration) -> Self {
        Listener { stream, pause: None, pause_duration: duration }
    }
}

impl Stream for Listener
{
    type Item = TcpStream;
    type Error = ();

    /// Polls for incoming connections, pausing if an error is encountered.
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.pause.as_mut().map(|p| p.poll()) {
            Some(Ok(Async::NotReady)) => return Ok(Async::NotReady),
            Some(Ok(Async::Ready(()))) | Some(Err(_)) => { self.pause.take(); }
            _ => ()
        }
        println!("poll data");
        match self.stream.poll() {
            Ok(x) => Ok(x),
            Err(e) => {
                println!("error accepting incoming connection: {}", e);
                self.pause = Some(Delay::new(Instant::now() + self.pause_duration));
                Err(())
            }
        }
    }
}

/// Stream that listens on an TCP/IP address.
#[derive(Debug)]
pub struct TcpListenStream {
    /// Stream of incoming sockets.
    inner: Listener,
    /// The port which we use as our listen port in listener event addresses.
    port: u16,
    /// Temporary buffer of listener events.
    pending: VecDeque<String>,

}

impl Stream for TcpListenStream {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Ok(Async::Ready(Some(event)))
            }

            let mut sock = match self.inner.poll() {
                Ok(Async::Ready(Some(sock))) => sock,
                Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) => {println!("err:{:?}",e); return Ok(Async::NotReady)},
                _ => return Ok(Async::NotReady)
            };
            println!("get sock");
            let sock_addr = match sock.peer_addr() {
                Ok(addr) => addr,
                Err(err) => {
                    println!("Failed to get peer address: {:?}", err);
                    continue
                }
            };
            let mut rd =  BytesMut::new();
            let res = sock.read_buf(&mut rd);
            println!("sock:{}, buf:{:?}", sock_addr, rd)

        }
    }
}

struct TcpCon {

}

impl TcpCon {
    fn listen_on(addr: &str) -> Result<TcpListenStream, String> {

        let socket_addr = addr.parse::<SocketAddr>().unwrap();
        let listener = tokio_tcp::TcpListener::bind(&socket_addr).unwrap();
        let local_addr = listener.local_addr().unwrap();
        let port = local_addr.port();


        let stream = TcpListenStream {
            inner: Listener::new(listener.incoming(), Duration::new(2,0)),
            port,
            pending: VecDeque::new(),
        };

        Ok(stream)
    }
}

fn tcp_listen_on() {
    let stream = TcpCon::listen_on("127.0.0.1:8765");
    let mut con = stream.unwrap();

    tokio::run(future::poll_fn(move || -> Result<_, ()> {
        loop {
            match con.poll().expect("Error while polling swarm") {
                Async::Ready(Some(e)) => println!("{:?}", e),
                Async::Ready(None) | Async::NotReady => {
                    return Ok(Async::NotReady)
                }
            }
        }
    }));
}

#[test]
fn tcp_server_test() {
    tcp_server();
}

#[test]
fn tcp_listen_test() {
    tcp_listen_on();
}