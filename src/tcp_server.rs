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
use futures::{future::Executor};

fn tcp_server() {
    let addr = "127.0.0.1:3000".parse().expect("couldn't parse address");
    let listener = TcpListener::bind(&addr).expect("couldn't bind address");
    let future = listener
        .incoming()
        .for_each(|socket| {
            let (reader, writer) = socket.split();
            write_all(writer, b"Welcome to the echo server\r\n")
                .and_then(|(writer, buf)| {
                    let out = std::str::from_utf8(buf);
                    println!("send data:{:?}", out);
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


struct Manager {
    tasks_handle: Vec<Box<dyn Future<Item = (), Error = ()> + Send>>,
}

impl Manager {
    fn new() -> Self {
        Self{tasks_handle: Vec::new()}
    }

    fn add_task(&mut self) -> (){
        let addr = "127.0.0.1:0".parse().expect("couldn't parse address");
        let listener = TcpListener::bind(&addr).expect("couldn't bind address");
        let local = listener.local_addr();
        println!("local addr:{:?}", local );
        let fut = Box::new(listener
            .incoming()
            .for_each(|socket| {
                let (reader, writer) = socket.split();
                write_all(writer, b"Welcome to the echo server\r\n")
                    .and_then(|(writer, buf)| {
                        println!("send data:{:?}", buf);
                        copy(reader, writer)
                            .map(|_| println!("Connection closed"))
                    })
            }).map_err(|e| eprintln!("An error occurred: {:?}", e)));
    //    let fut = Box::new(futures::future::ok(3).and_then(|data|{println!("execute:{}", data); futures::future::ok::<(),()>(())}));
        self.tasks_handle.push(fut);
    }

    fn poll(&mut self) -> Async<u32> {
        let executor = tokio_executor::DefaultExecutor::current();
//        let iter = self.tasks_handle.into_iter();
//        //let fut = futures::future::ok(3).and_then(|data|{println!("execute:{}", data); futures::future::ok(())});
//        iter.for_each( move |fut| executor.execute(Box::into_raw(fut)));
//        self.tasks_handle.clear();
        for to_spawn in self.tasks_handle.drain(..) {
            executor.execute(to_spawn);
        }
        Async::Ready(0)
    }
}

struct Colletion  {
    inner: Manager
}

impl Colletion {
    fn new(manager: Manager) -> Self {
        Colletion{inner: manager}
    }

    fn poll(&mut self) -> Async<u32>{
        self.inner.poll()
    }
}



/// Stream that listens on an TCP/IP address.
///#[derive(Debug)]
pub struct TcpListenStream {
    /// Stream of incoming sockets.
    inner: Listener,
    /// The port which we use as our listen port in listener event addresses.
    port: u16,
    /// Temporary buffer of listener events.
    pending: VecDeque<String>,
    colletion: Colletion,

}

impl Stream for TcpListenStream {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        loop {
            (self).colletion.poll();
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
            self.colletion.inner.add_task();
            let sock_addr = match sock.peer_addr() {
                Ok(addr) => addr,
                Err(err) => {
                    println!("Failed to get peer address: {:?}", err);
                    continue
                }
            };
            //let mut rd =  BytesMut::new();
            //let res = sock.read_buf(&mut rd);
            let (reader, writer) = sock.split();
            tokio::spawn(write_all(writer, b"Welcome to the echo server\r\n")
                .and_then(move|(writer, buf)| {
                    //copy(reader, writer).map(|_| println!("Connection closed"))
                    //reader.
                    futures::future::ok(())
                }).map_err(|e| eprintln!("Error occured: {:?}", e))
 //
                );
            println!("sock:{}, buf", sock_addr);
            self.pending.push_back(sock_addr.to_string());


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
            colletion: Colletion::new(Manager::new())
        };

        Ok(stream)
    }
}

fn tcp_listen_on() {
    let stream = TcpCon::listen_on("127.0.0.1:8765");
    let mut con = stream.unwrap();

    //let executor = tokio_executor::DefaultExecutor::current();
    tokio::run(future::poll_fn(move || -> Result<_, ()> {
    //let res = executor.execute(future::poll_fn(move || -> Result<_, ()> {
        println!("enter loop");
        loop {
            match con.poll().expect("Error while polling swarm") {
                Async::Ready(Some(e)) => println!("resp:{:?}", e),
                Async::Ready(None) | Async::NotReady => {
                    println!("con poll not ready");
                    return Ok(Async::NotReady)
                }
            }
        }
    }));
    //println!("execute:{:?}", res);
}

#[test]
fn tcp_server_test() {
    tcp_server();
}

//nc 127.0.0.1 8765 or in mac nc -p 4567 127.0.0.1 8765
#[test]
fn tcp_listen_test() {
    tcp_listen_on();
}