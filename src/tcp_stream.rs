use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;


fn tcp_client() {
    // Parse the address of whatever server we're talking to
    let addr = "127.0.0.1:6142".parse().unwrap();
    let client = TcpStream::connect(&addr).and_then(|stream| {
        println!("created stream");

        io::write_all(stream, "hello world\n").and_then(|result| {
            println!("wrote to stream; success={:?}", result);
            Ok(())
        })
    }).map_err(|err| {
        // All tasks must have an `Error` type of `()`. This forces error
        // handling and helps avoid silencing failures.
        //
        // In our example, we are only going to log the error to STDOUT.
        println!("connection error = {:?}", err);
    });

    println!("About to create the stream and write to it...");
    tokio::run(client);
    println!("Stream has been created and written to.");
}


fn tcp_read() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let echo_fut = TcpStream::connect(&addr)
        .and_then(|stream| {
            // We're going to read the first 32 bytes the server sends us
            // and then just echo them back:
            let mut buf = vec![0; 2];
            // First, we need to read the server's message
            tokio::io::read_exact(stream, buf)
        })
        .and_then(|(stream, buf)| {
            // Then, we use write_all to write the entire buffer back:
            println!("read:{:?}",buf);
            tokio::spawn(tokio::io::write_all(stream, b"Welcome to the echo client\r\n")
                .and_then(move|(writer, buf)| {

                future::ok(())
                //reader.
            }).map_err(|e| eprintln!("Error occured: {:?}", e)));
            futures::future::ok(())
        }).map_err(|err| {
        // All tasks must have an `Error` type of `()`. This forces error
        // handling and helps avoid silencing failures.
        //
        // In our example, we are only going to log the error to STDOUT.
        println!("connection error = {:?}", err);
    });
   /*     .inspect(|(_stream, buf)| {
            println!("echoed back {} bytes: {:x?}", buf.len(), buf);
           // future::ok(())
        });*/
    tokio::run(echo_fut);
}
#[test]
fn tcp_client_test() {
    tcp_client();
}

#[test]
fn tcp_read_test() {
    tcp_read();
}