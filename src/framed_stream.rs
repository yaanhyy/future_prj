use bytes::{BufMut, BytesMut};
use std::{fmt::{Debug, Display}, io, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, str};
use tokio::{
    codec::{BytesCodec, Framed, Encoder, Decoder},
    net::{TcpListener, TcpStream}};
use futures::{stream};
use futures::{future::{self}, prelude::*};
pub struct LinesCodec {
    // Stored index of the next index to examine for a `\n` character.
    // This is used to optimize searching.
    // For example, if `decode` was called with `abc`, it would hold `3`,
    // because that is the next index to examine.
    // The next time `decode` is called with `abcde\n`, the method will
    // only look at `de\n` before returning.
    next_index: usize,
}

impl LinesCodec {
    fn new() -> Self {
        LinesCodec{next_index:0}
    }
}

impl Decoder for LinesCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        // Look for a byte with the value '\n' in buf. Start searching from the search start index.
        if let Some(newline_offset) = buf[self.next_index..].iter().position(|b| *b == b'\n')
        {
            // Found a '\n' in the string.

            // The index of the '\n' is at the sum of the start position + the offset found.
            let newline_index = newline_offset + self.next_index;
            println!("get incoming:{}", newline_offset);
            // Split the buffer at the index of the '\n' + 1 to include the '\n'.
            // `split_to` returns a new buffer with the contents up to the index.
            // The buffer on which `split_to` is called will now start at this index.
            let line = buf.split_to(newline_index + 1);

            // Trim the `\n` from the buffer because it's part of the protocol,
            // not the data.
            let line = &line[..line.len() - 1];

            // Convert the bytes to a string and panic if the bytes are not valid utf-8.
            let line = str::from_utf8(&line).expect("invalid utf8 data");

            // Set the search start index back to 0.
            self.next_index = 0;
            println!("line:{}",line);
            // Return Ok(Some(...)) to signal that a full frame has been produced.
            Ok(Some(line.to_string()))
        } else {
            // '\n' not found in the string.

            // Tell the next call to start searching after the current length of the buffer
            // since all of it was scanned and no '\n' was found.
            self.next_index = buf.len();
            println!("next_index:{}",self.next_index);
            // Ok(None) signifies that more data is needed to produce a full frame.
            Ok(None)
        }
    }
}

impl Encoder for LinesCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, line: Self::Item, buf: &mut BytesMut) -> Result<(), io::Error> {
        println!("encode:{}", line);
        // It's important to reserve the amount of space needed. The `bytes` API
        // does not grow the buffers implicitly.
        // Reserve the length of the string + 1 for the '\n'.
        buf.reserve(line.len() + 1);

        // String implements IntoBuf, a trait used by the `bytes` API to work with
        // types that can be expressed as a sequence of bytes.
        buf.put(line);

        // Put the '\n' in the buffer.
        buf.put_u8(b'\n');

        // Return ok to signal that no error occured.
        Ok(())
    }
}

fn frame_process(port: u16) {
    let i = Ipv4Addr::new(127, 0, 0, 1);
    let addr = SocketAddr::V4(SocketAddrV4::new(i, port));

    let future = TcpStream::connect(&addr).and_then(|sock| {
        let (mut framed_sink, framed_stream )= Framed::new(sock, LinesCodec::new()).split();
        //tokio::spawn(poll_fn(move||{ framed_sink.start_send(); Ok(())}));
        framed_stream.for_each(move |line| {
            println!("Received line {}", line);
           // framed_sink.start_send(line) ;
            Ok(())
        })
    });
    future.wait();
    //tokio::run(future);
}

fn frame_echo(port: u16) {
    let i = Ipv4Addr::new(127, 0, 0, 1);
    let addr = SocketAddr::V4(SocketAddrV4::new(i, port));

    let future = TcpStream::connect(&addr).and_then(|sock| {
        let framed_stream = Framed::new(sock, LinesCodec::new());
        //tokio::spawn(poll_fn(move||{ framed_sink.start_send(); Ok(())}));
        let mut f = framed_stream.into_future();
        loop {

            let res = f.poll();
            match res {
                Ok(Async::Ready((res, frame))) => {
                    println!("res:{:?}", res);
                    frame.send(res.unwrap());
                    return Ok(())
                },
                Err((t, frame)) => {},
                Ok(Async::NotReady)=> {
                    println!("not ready");
                }
            };
        }
        Ok(())
    }) .map_err(|e| eprintln!("Error reading directory: {}", e));
    //future.wait();
    tokio::run(future);
}


#[test]
fn frame_process_test() {
    frame_process(7865);
}

#[test]
fn frame_echo_test() {
    frame_echo(7866);
}