use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

fn echo() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    // Here we convert the `TcpListener` to a stream of incoming connections
    // with the `incoming` method. We then define how to process each element in
    // the stream with the `for_each` combinator method
    let server = listener.incoming().for_each(|socket| {
        // TODO: Process socket
        let (reader, writer) = socket.split();
        // copy bytes from the reader into the writer
        let amount = io::copy(reader, writer);

        let msg = amount.then(|result| {
            match result {
                Ok((amount, _, _)) => println!("wrote {} bytes", amount),
                Err(e)             => println!("error: {}", e),
            }


            Ok(())
        });

        // spawn the task that handles the client connection socket on to the
        // tokio runtime. This means each client connection will be handled
        // concurrently
        tokio::spawn(msg);
        Ok(())
    }).map_err(|err| {
            // Handle error by printing to STDOUT.
            println!("accept error = {:?}", err);
    });

    println!("server running on localhost:6142");
    // `select` completes when the first of the two futures completes. Since
    // future::ok() completes immediately, the server won't hang waiting for
    // more connections. This is just so the doc test doesn't hang.
    //let server = server.select(futures::future::ok(())).then(|_| Ok(()));

    // Start the server
    //
    // This does a few things:
    //
    // * Start the Tokio runtime
    // * Spawns the `server` task onto the runtime.
    // * Blocks the current thread until the runtime becomes idle, i.e. all
    //   spawned tasks have completed.
    tokio::run(server);
}

#[test]
fn echo_test() {
    echo();
}
