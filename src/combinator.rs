use futures::Stream;
use std::time::Duration;
use tokio::timer::Interval;
use tokio::prelude::*;
use futures_util::future::FutureExt;

struct Fibonacci {
    curr: u64,
    next: u64,
}

fn fibonacci() {
    let mut fib = Fibonacci { curr: 1, next: 1 };

    let future = Interval::new_interval(Duration::from_secs(1)).map(move |x| {
        println!("instant:{:?}", x);
        let curr = fib.curr;
        let next = curr + fib.next;

        fib.curr = fib.next;
        fib.next = next;

        curr
    });

    tokio::run(future.take(10).map_err(|_| ()).for_each(|num| {
        println!("{}", num);
        Ok(())
    }));
}



fn stream_to_future() {
    let a = Ok::<_,()>(1). into_future();
    let res = a.and_then(|a|{Ok(a+3)}).wait().unwrap();
    println!("res:{}", res);
}

#[test]
fn fibonacci_test() {
    fibonacci();
}

#[test]
fn stream_to_future_test() {
    stream_to_future();
}

