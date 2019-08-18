extern crate tokio;
extern crate futures;
use futures::sync::oneshot;
use futures::future::lazy;
use futures::sync::mpsc;
use futures::{stream, Future, Stream, Sink};

fn spawn_task() {
    (0..4).flat_map(|x| x * 100 .. x * 110)
        .enumerate()
     //   .filter(|&(i, x)| (i + x) % 3 == 0)
        .for_each(|(i, x)| println!("{}:{}", i, x));

    tokio::run(lazy(|| {
        for i in 0..4 {
            tokio::spawn(lazy(move || {
                println!("Hello from task {}", i);
                Ok(())
            }));
        }

        Ok(())
    }));

    tokio::run(lazy(|| {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(lazy(|| {
            tx.send("hello from spawned task");
            Ok(())
        }));

        rx.and_then(|msg| {
            println!("Got `{}`", msg);
            Ok(())
        })
            .map_err(|e| println!("error = {:?}", e))
    }));

    tokio::run(lazy(|| {
        let (tx, rx) = mpsc::channel(1_024);

        tokio::spawn({
            stream::iter_ok(0..10).fold(tx, |tx, i| {
                tx.send(format!("Message {} from spawned task", i))
                    .map_err(|e| println!("error = {:?}", e))
            })
                .map(|_| ()) // Drop tx handle
        });

        rx.for_each(|msg| {
            println!("Got `{}`", msg);
            Ok(())
        })
    }));
}

#[test]
fn spawn_task_test() {
    spawn_task();
}