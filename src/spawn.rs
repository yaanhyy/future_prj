extern crate tokio;
extern crate futures;

use futures::future::lazy;

fn spawn_task() {
    tokio::run(lazy(|| {
        for i in 0..4 {
            tokio::spawn(lazy(move || {
                println!("Hello from task {}", i);
                Ok(())
            }));
        }

        Ok(())
    }));
}

#[test]
fn spawn_task_test() {
    spawn_task();
}