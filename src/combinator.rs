use futures::Stream;
use std::time::Duration;
use tokio::timer::Interval;

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

#[test]
fn fibonacci_test() {
    fibonacci();
}

