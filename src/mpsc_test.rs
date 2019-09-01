use bytes::{BufMut, Bytes, BytesMut};
use futures::future::{self, Either};
use futures::sync::mpsc;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use std::{pin::Pin, time::{Duration, Instant}};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::timer::Interval;
use std::thread::{sleep};

struct Task {
    tx: mpsc::UnboundedSender<u128>,
    interval: Interval,
}

impl Stream for Task {
    type Item = u128;
    type Error = ();
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        loop {
            let res = self.interval.poll().map_err(|_| ());
            if res.is_ok() {
                println!("ins:{:?}",res);
                if let Ok(Async::Ready(t)) = res {
                    let ins: Instant = t.unwrap();
                    let ins = ins.elapsed().as_millis();
                    println!("send:{}",ins);
                    self.tx.unbounded_send(ins );
                    break;
                } else {
                    return Ok(Async::NotReady)
                }
            }

        }
        Ok(Async::NotReady)
    }
}

struct Front {
    rx: mpsc::UnboundedReceiver<u128>,
    index: u128,
    stream: Task,
}

impl  Stream for Front {
    type Item = u128;
    type Error = ();
    fn poll(& mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let res = (self.rx.poll());
        sleep(Duration::from_secs(1));
        if res.is_ok() {
            if let Ok(Async::Ready(t)) = res {
                self.index = t.unwrap();
                return Ok(Async::Ready(Some(self.index)))
            } else {
                self.stream.poll();
                return Ok(Async::NotReady)
            }
        } else {
            Ok(Async::NotReady)
        }
    }

}

fn start_task() {
    let (tx, rx ) = mpsc::unbounded();
    let duration = Duration::from_secs(3);
    let task = Task{tx, interval: Interval::new(Instant::now(), duration)};
    let front = Front{rx: rx, index: 0, stream: task};
//    let task_fut = task.for_each(|index| {
//        println!("task index:{}", index);
//        futures::future::ok(())
//    }).map_err(|err| {
//        println!("message error = {:?}", err);
//    });


    let future_front = front.for_each(|index| {
        println!("fron index:{}", index);
        futures::future::ok(())
    }).map_err(|err| {
        println!("message error = {:?}", err);
    });
    tokio::run(future_front);
   // tokio::run(task_fut.join(future_front).map(|_| ()));


}

#[test]
fn start_task_test() {
    start_task();
}