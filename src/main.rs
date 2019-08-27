//#![feature(conservative_impl_trait, proc_macro, generators)]

//extern crate futures_await as futures;
//extern crate tokio_core;
#![feature(async_await)]
#[macro_use]
extern crate futures;
use futures::done;
use futures::future;
use futures::future::{err, ok};
use tokio_core::reactor::Core;
use std::error::{self,Error};
use std::fmt;
use tokio::prelude::*;
extern crate chrono;
use chrono::prelude::*;
use chrono::Duration;

mod spawn;
mod tcp_stream;
mod echo;
mod tcp_test;
mod tcp_server;
mod async_future;
mod stream_sync;
mod stream_async;
mod backend;
mod http_server;
mod simple_future;

fn my_fn() -> Result<u32, Box<Error>> {
    Ok(100)
}

fn my_fut() -> impl Future<Item = u32,  Error= Box<Error + 'static>> {
    ok(100)
}

fn my_fn_squared(i: u32) -> Result<u32, Box<Error>> {
    Ok(i * i)
}


fn my_fut_squared(i: u32) -> impl Future<Item = u32,  Error= Box<Error+ 'static> > {
    ok(i * i)
}

fn fn_plain(i: u32) -> u32 {
    i - 50
}

fn fut_generic_own<A>(a1: A, a2: A) -> impl Future<Item = A, Error = Box<Error+ 'static>>
    where
        A: std::cmp::PartialOrd,
{
    if a1 < a2 {
        ok(a1)
    } else {
        ok(a2)
    }
}

#[derive(Debug, Default)]
pub struct ErrorA {}

impl fmt::Display for ErrorA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ErrorA!")
    }
}

impl error::Error for ErrorA {
    fn description(&self) -> &str {
        "Description for ErrorA"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug, Default)]
pub struct ErrorB {}

impl fmt::Display for ErrorB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ErrorB!")
    }
}

impl error::Error for ErrorB {
    fn description(&self) -> &str {
        "Description for ErrorB"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

fn fut_error_a() -> impl Future<Item = (), Error = ErrorA> {
    err(ErrorA {})
}

fn fut_error_b() -> impl Future<Item = (), Error = ErrorB> {
    err(ErrorB {})
}

fn my_fut_ref<'a>(s: &'a str) -> impl Future<Item = &'a str, Error = Box<Error>> +'a {
    ok(s)
}


fn my_fut_ref_chained<'a>(s: &'a str) -> impl Future<Item = String, Error = Box<Error>> +'a {
    my_fut_ref(s).and_then(|s| ok(format!("received == {}", s)))
}

#[derive(Debug)]
struct WaitForIt {
    message: String,
    until: DateTime<Utc>,
    polls: u64,
}

impl WaitForIt {
    pub fn new(message: String, delay: Duration) -> WaitForIt {
        WaitForIt {
            polls: 0,
            message: message,
            until: Utc::now() + delay,
        }
    }
}

impl Future for WaitForIt {
    type Item = String;
    type Error = Box<Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let now = Utc::now();
        thread::sleep(::std::time::Duration::from_millis(500));
        if self.until < now {
            Ok(Async::Ready(
                format!("{} after {} polls!", self.message, self.polls),
            ))
        } else {
            self.polls += 1;

            println!("not ready yet --> {:?}", self);
            if self.message.eq(&String::from("I'm done:")) {
                futures::task::current().notify();
            }
            Ok(Async::NotReady)
        }
    }
}
use std::thread;
pub struct WaitInAnotherThread {
    end_time: DateTime<Utc>,
    msg : String,
    running: bool,
}

impl WaitInAnotherThread {
    pub fn
    new(how_long: Duration, msg :String) -> WaitInAnotherThread {
        WaitInAnotherThread {
            end_time: Utc::now() + how_long,
            msg : msg,
            running: false,
        }
    }

    fn run(&mut self, task: task::Task) {
        let lend = self.end_time;
        println!("msg: {:?}!", self.msg);
        thread::spawn(move || {
            while Utc::now() < lend {
                let delta_sec = lend.timestamp() - Utc::now().timestamp();
                if delta_sec > 0 {
                    thread::sleep(::std::time::Duration::from_secs(delta_sec as u64));
                }
                task.notify();
            }

        });
    }
}

use futures::task;

impl Future for WaitInAnotherThread {
    type Item = ();
    type Error = ();



    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if Utc::now() < self.end_time {
            println!("not ready yet! parking the task.");

            if !self.running {
                println!("side thread not running! starting now!");
                self.run(task::current());
                self.running = true;
            }

            Ok(Async::NotReady)
        } else {
            println!("ready! the task will complete.");
            Ok(Async::Ready(()))
        }
    }
}

use futures::future::join_all;


struct MyStream {
    current: u32,
    max: u32,
    core_handle : tokio_core::reactor::Handle,
}

impl MyStream {
    pub fn new(max: u32, handle: tokio_core::reactor::Handle) -> MyStream {
        MyStream {
            current: 0,
            max: max,
            core_handle : handle,
        }
    }
}

impl Stream for MyStream {
    type Item = u32;
    type Error = Box<Error>;

//    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
//        match self.current {
//            ref mut x if *x < self.max => {
//                *x = *x + 1;
//
//                Ok(Async::Ready(Some(*x)))
//            }
//            _ => Ok(Async::Ready(None)),
//        }
//    }

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use futures::future::Executor;

        match self.current {
            ref mut x if *x < self.max => {
                *x = *x + 1;
                self.core_handle.execute(WaitInAnotherThread::new(
                    Duration::seconds(2),
                    format!("WAIT {:?}", x),
                ));
                Ok(Async::Ready(Some(*x)))
            }
            _ => Ok(Async::Ready(None)),
        }
    }
}
use tokio::fs;
use std::env::args;

use tokio::net::TcpStream;
use tokio::prelude::*;
use std::net::ToSocketAddrs;
fn main() {
    let mut addr_iter = "httpbin.org:80".to_socket_addrs().unwrap();
    let addr = match addr_iter.next() {
        None => panic!("DNS resolution failed"),
        Some(addr) => addr,
    };
    let future = TcpStream::connect(&addr)
        .map_err(|e| eprintln!("Error connecting: {:?}", e))
        .map(|stream| {
            println!("Got a stream: {:?}", stream);
        });
    tokio::run(future);

//    let future = fs::read_dir(".")
//        .map_err(|e| eprintln!("Error reading directory: {}", e))
//        .and_then(|readdir| {
//            println!("readdir:{:?}", readdir);
//            readdir
//                .map_err(|e| eprintln!("Error reading directory: {}", e))
//                .for_each(|entry| {
//                    println!("{:?}", entry.path());
//                    futures::future::ok(())
//                })
//        })
//        ;
//    tokio::run(future);
    let future = fs::read_dir(".")
        .and_then(|readdir| {
            readdir
                .for_each(|entry| {
                    println!("{:?}", entry.path());
                    future::ok(())
                })
        })
        .map_err(|e| eprintln!("Error reading directory: {}", e));
    tokio::run(future);

    let future = fs::read_dir("/mnt")
        .flatten_stream()
        .for_each(|entry| {
            println!("flatten_stream:{:?}", entry.path());
            futures::future::ok(())
        })
        .map_err(|e| eprintln!("Error reading directory: {}", e))
        ;
    tokio::run(future);


    let args_vec: Vec<String> = args().skip(1).collect();
    println!("{:?}", args_vec);
    let future =
        stream::iter_ok(args_vec).for_each(|dir| {
            println!("{:?}", dir);
            let read_future = fs::read_dir(dir)
                .flatten_stream().map_err(|e| eprintln!("Error reading directory: {}", e))
                .for_each(|entry| {
                    println!("{:?}", entry.path());
                    futures::future::ok(())
                });
             //   .map_err(|e| eprintln!("Error reading directory: {:?}", e));
            //read_future
            tokio::spawn(read_future)
        });

    tokio::run(future);

    println!("for read args");
    let future = futures::future::ok(()).and_then(|_| {
        for dir in args().skip(1) {
            let future = fs::read_dir(dir)
                .flatten_stream()
                .map_err(|e| eprintln!("Error reading directory: {}", e))
                .for_each(|entry| {
                    println!("{:?}", entry.path());
                    futures::future::ok(())
                });

           // future
            tokio::spawn(future);

        }
        futures::future::ok(())
    });
    tokio::run(future);

    println!("poll_fn");
    let mut future = future::poll_fn(|| {
        for dir in args().skip(1) {
            let future = fs::read_dir(dir)
                .flatten_stream()
                .map_err(|e| eprintln!("Error reading directory: {}", e))
                .for_each(|entry| {
                    println!("{:?}", entry.path());
                    future::ok(())
                })
                ;
            tokio::spawn(future);
        }
        //Ok(Async::NotReady)
        Ok(Async::Ready(()))
    });

    tokio::run(future);

    //stream
    let mut reactor = Core::new().unwrap();
    let my_stream = MyStream::new(5, reactor.handle());

    // we use for_each to consume
    // the stream
    let fut = my_stream.for_each(|num| {
        println!("num === {:?}", num);
        ok(())
    });

    // this is a manual future. it's the same as the
    // future spawned into our stream
    let wait = WaitInAnotherThread::new(Duration::seconds(3), "Manual3".to_owned());

    // we join the futures to let them run concurrently
    let future_joined = fut.map_err(|err| {}).join(wait);

    // let's run the future
    let ret = reactor.run(future_joined).unwrap();
    println!("stream ret == {:?}", ret);

//    let my_stream = MyStream::new(5);
//
//    let fut = my_stream.for_each(|num| {
//        println!("num === {}", num);
//        ok(())
//    });

    // let's run the future
//    let ret = reactor.run(fut).unwrap();
//    println!("ret == {:?}", ret);

    //future chain
    let wait = WaitInAnotherThread::new(Duration::seconds(3), String::from("chain"));
    println!("wait future started");
    let ret = reactor.run(wait).unwrap();
    println!("wait future completed. ret == {:?}", ret);

    let wfi_1 = WaitForIt::new("I'm done:".to_owned(), Duration::seconds(1));
    println!("wfi_1 == {:?}", wfi_1);
    //let ret = reactor.run(wfi_1).unwrap();
    //println!("ret == {:?}", ret);

    let wfi_2 = WaitForIt::new("I'm done too:".to_owned(), Duration::seconds(1));
    println!("wfi_2 == {:?}", wfi_2);

    //select_all
   //let v = vec![wfi_1, wfi_2];
   // let sel = futures::future::select_all(v);

    //select2
    let sel = wfi_1.select2(wfi_2);

    //join_all
    //let sel = join_all(v);

    let ret = reactor.run(sel).unwrap();
    println!("ret == {:?}", ret);




    let retval = my_fn().unwrap();
    println!("{:?}", retval);


    let retval2 = my_fn_squared(retval).unwrap();
    println!("{:?}", retval2);

    let mut reactor = Core::new().unwrap();

//    let retval = reactor.run(my_fut()).unwrap();
//    println!("{:?}", retval);
//
//    let retval2 = reactor.run(my_fut_squared(retval)).unwrap();
//    println!("{:?}", retval2);

    let chained_future = my_fut().and_then(|retval| my_fn_squared(retval));
    let retval2 = reactor.run(chained_future).unwrap();
    println!("{:?}", retval2);

    let chained_future = my_fut().and_then(|retval| {
        let retval2 = fn_plain(retval);
        my_fut_squared(retval2)
    });
    let retval3 = reactor.run(chained_future).unwrap();
    println!("{:?}", retval3);

    let chained_future = my_fut().and_then(|retval| {
        done(my_fn_squared(retval)).and_then(|retval2| my_fut_squared(retval2))
    });
    let retval3 = reactor.run(chained_future).unwrap();
    println!("{:?}", retval3);

    let future = fut_generic_own("Sampdoria", "Juventus");
    let retval = reactor.run(future).unwrap();
    println!("fut_generic_own == {}", retval);

    let retval = reactor.run(fut_error_a()).unwrap_err();
    println!("fut_error_a == {:?}", retval);

    let retval = reactor.run(fut_error_b()).unwrap_err();
    println!("fut_error_b == {:?}", retval);

    let future = fut_error_a()
        .map_err(|e| {
            println!("mapping {:?} into ErrorB", e);
            ErrorB::default()
        })
        .and_then(|_| fut_error_b())
        .map_err(|e| {
            println!("mapping {:?} into ErrorA", e);
            ErrorA::default()
        })
        .and_then(|_| fut_error_a());

    let retval = reactor.run(future).unwrap_err();
    println!("error chain == {:?}", retval);

    let retval = reactor
        .run(my_fut_ref_chained("str with lifetime"))
        .unwrap();
    println!("my_fut_ref_chained == {}", retval);

}
