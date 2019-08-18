extern crate tokio;
extern crate futures;

use futures::future::lazy;

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
}

#[test]
fn spawn_task_test() {
    spawn_task();
}