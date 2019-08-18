use futures::{Stream, Poll, Async};
use std::fmt;
use futures::prelude::*;

pub struct Display10<T> {
    stream: T,
    curr: usize,
}

impl<T> Display10<T> {
    fn new(stream: T) -> Display10<T> {
        Display10 {
            stream,
            curr: 0,
        }
    }
}

impl<T> Future for Display10<T>
    where
        T: Stream,
        T::Item: fmt::Display,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        while self.curr < 10 {
            let value = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                // There were less than 10 values to display, terminate the
                // future.
                None => break,
            };

            println!("value #{} = {}", self.curr, value);
            self.curr += 1;
        }

        Ok(Async::Ready(()))
    }
}

pub struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Fibonacci {
        Fibonacci {
            curr: 1,
            next: 1,
        }
    }
}

impl Stream for Fibonacci {
    type Item = u64;

    // 该流将永远不会产生错误
    type Error = ();

    fn poll(&mut self) -> Poll<Option<u64>, ()> {
        let curr = self.curr;
        let next = curr + self.next;

        self.curr = self.next;
        self.next = next;

        Ok(Async::Ready(Some(curr)))
    }
}

fn sync_stream() {
    let fib = Fibonacci::new();
    let display = Display10::new(fib);

    tokio::run(display);
}

#[test]
fn sync_stream_test() {
    sync_stream();
}