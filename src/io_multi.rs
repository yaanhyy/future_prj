use std::{fmt::{Debug, Display}, io, net::{Ipv4Addr, SocketAddr, SocketAddrV4}};
use futures::{future::{self, Either, Loop}, prelude::*};
use tokio::{
    codec::{BytesCodec, Framed, Encoder, Decoder},
    net::{TcpListener, TcpStream},
    runtime::Runtime
};
use yamux::{Config, Connection, Mode, StreamHandle, State};

const PORT:u16 = 4765;

fn server()  -> TcpListener {
    let cfg = Config::default();
    let i = Ipv4Addr::new(127, 0, 0, 1);
    let s = SocketAddr::V4(SocketAddrV4::new(i, PORT));
    let l = TcpListener::bind(&s).unwrap();
    l
}

fn client() -> impl Future<Item = Connection<TcpStream>, Error = ()>  {
    let cfg = Config::default();
    let i = Ipv4Addr::new(127, 0, 0, 1);
    let s = SocketAddr::V4(SocketAddrV4::new(i, PORT));
    let codec = BytesCodec::new();

    let stream = TcpStream::connect(&s)
        .map_err(|e| println!("connect failed: {}", e))
        .map(move |sock| {println!("client_map");Connection::new(sock, cfg, Mode::Client)});
    stream
}

#[test]
fn io_test() {
    let cfg = Config::default();
    let codec = BytesCodec::new();
    let l = server();
    let server = l.incoming()
        .map(move |sock| Connection::new(sock, cfg.clone(), Mode::Server))
        .into_future()
        .map_err(|(e, _rem)| println!("accept failed: {}", e))
        .and_then(|(maybe, _rem)| {println!("server_map");maybe.ok_or(())});
    let server = server.and_then(|c| {
        match c.open_stream() {
            Ok(Some(s)) => {
                println!("C: new stream: {:?}", stream);
                let codec = Framed::new(stream, d.clone());
                let future = codec.send(msg)
                    .map_err(|e| error!("C: send error: {}", e))
                    .and_then(move |codec| {
                        codec.collect().and_then(move  |data| {
                            println!("C: received {:?}", data);
                            v.extend(data);
                            Ok(Loop::Continue((v, it)))
                        })
                            .map_err(|e| error!("C: receive error: {}", e))
                    });
            },
            Ok(None) => future::ok(),

            Err(e) => future::ok()
        }
    });

    let c = client();
    let stream = c.and_then(|x| {
        x.for_each(move |stream| {
            let (stream_out, stream_in) = Framed::new(stream, codec).split();
            stream_in
                .take(1)
                .map(|frame_in| frame_in.into())
                .forward(stream_out)
                .from_err()
                .map(|_| ())
        })
    });

    let mut rt = Runtime::new().unwrap();
    rt.spawn(server);
    stream.wait().unwrap();
}