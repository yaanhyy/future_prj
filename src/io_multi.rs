use std::{fmt::{Debug, Display}, io, net::{Ipv4Addr, SocketAddr, SocketAddrV4}};
use futures::{future::{self, Either, Loop}, prelude::*};
use tokio::{
    codec::{BytesCodec, Framed, Encoder, Decoder},
    net::{TcpListener, TcpStream},
    runtime::Runtime
};
use yamux::{Config, Connection, Mode, StreamHandle, State};
use bytes::Bytes;
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
    let d = BytesCodec::new();
    let l = server();
    let serv = l.incoming()
        .map(move |sock| Connection::new(sock, cfg.clone(), Mode::Server))
        .into_future()
        .map_err(|(e, _rem)| println!("accept failed: {}", e))
        .and_then(|(maybe, _rem)| {println!("server_map");maybe.ok_or(())});
    let server = serv.and_then(move |c| {
        match c.open_stream() {
            Ok(Some(stream)) => {
                println!("C: new stream: {:?}", stream);
                let codec = Framed::new(stream, d.clone());
                let future = codec.send(Bytes::from(vec![48,49,50]))
                    .map_err(|e| println!("C: send error: {}", e))
                    .and_then(move |codec| {
                        codec.collect().and_then(move  |data| {
                            println!("C: received {:?}", data);
                            //v.extend(data);
                            Ok(())
                        })
                            .map_err(|e| println!("C: receive error: {}", e))
                    });
                Either::A(future)
            },
            Ok(None) => Either::B(future::ok(())),

            Err(e) => Either::B(future::ok(()))
        }
    });

    let c = client();
    let stream = c.and_then(move|x| {
        x.for_each(move |stream| {
            let (stream_out, stream_in) = Framed::new(stream, d).split();
            stream_in
                .take(1)
                .map(|frame_in| frame_in.into())
                .forward(stream_out)
                .from_err()
                .map(|_| ())
            //future::ok(())
        }).map_err(|e| println!("S: connection error: {}", e))
    });

    let mut rt = Runtime::new().unwrap();
    rt.spawn(stream);
    server.wait().unwrap();
    //server1.wait().unwrap();
}