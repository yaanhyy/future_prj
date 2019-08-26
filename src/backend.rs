use tokio::io;
use tokio::net::TcpListener;
use tokio::timer::Interval;
use futures::{future, stream, Future, Stream, Sink};
use futures::future::{lazy};
use futures::sync::{mpsc, oneshot};
use std::time::{Duration,Instant};
use std::thread::{sleep, spawn};

// 定义后台任务。`rx` 参数是通道的接收句柄。
// 任务将从通道中拉取 `usize` 值（代表套接字读取都字节数）并求总和。
// 所求的总和将每三十秒被打印到标准输出（stdout）然后重置为零。
fn bg_task(rx: mpsc::Receiver<usize>)
           -> impl Future<Item = (), Error = ()>
{
    // The stream of received `usize` values will be merged with a 30
    // second interval stream. The value types of each stream must
    // match. This enum is used to track the various values.
    #[derive(Eq, PartialEq)]
    enum Item {
        Value(usize),
        Tick,
        Done,
    }

    // 打印总和到标准输出都时间间隔。
    let tick_dur = Duration::from_secs(30);

    let interval = Interval::new_interval(tick_dur)
        .map(|_| Item::Tick)
        .map_err(|_| ());

    // 将流转换为这样的序列:
    // Item(num), Item(num), ... Done
    //
    let items = rx.map(Item::Value)
        .chain(stream::once(Ok(Item::Done)))
        // Merge in the stream of intervals
        .select(interval)
        // Terminate the stream once `Done` is received. This is necessary
        // because `Interval` is an infinite stream and `select` will keep
        // selecting on it.
        .take_while(|item| future::ok(*item != Item::Done));

    // With the stream of `Item` values, start our logic.
    //
    // Using `fold` allows the state to be maintained across iterations.
    // In this case, the state is the number of read bytes between tick.
    items.fold(0, |num, item| {
        match item {
            // Sum the number of bytes with the state.
            Item::Value(v) => future::ok(num + v),
            Item::Tick => {
                println!("bytes read = {}", num);

                // 重置字节计数器
                future::ok(0)
            }
            _ => unreachable!(),
        }
    })
        .map(|_| ())
}

fn backend() {
// 启动应用
    tokio::run(lazy(|| {
        let addr = "127.0.0.1:1234".parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

// 创建用于与后台任务通信的通道。
        let (tx, rx) = mpsc::channel(1_024);

// 创建后台任务：
        tokio::spawn(bg_task(rx));

        listener.incoming().for_each(move |socket| {
// 接收到一个入站套接字。
//
// 创建新任务处理套接字。
            tokio::spawn({
// 每个新创建的任务都会拥有发送者句柄的一份拷贝。
                let tx = tx.clone();

// 在本例中，将 "hello world" 写入套接字然后关闭之。
                io::read_to_end(socket, vec![])
// 释放套接字
                    .and_then(move |(_, buf)| {
                        tx.send(buf.len())
                            .map_err(|_| io::ErrorKind::Other.into())
                    })
                    .map(|_| ())
// 打印错误信息到标准输出
                    .map_err(|e| println!("socket error = {:?}", e))
            });

// 接收下一个入站套接字
            Ok(())
        })
            .map_err(|e| println!("listener error = {:?}", e))
    }));
}

type Message = oneshot::Sender<Duration>;
use futures::Poll;
use futures::Async;
use rand;
use rand::Rng;
// My code
struct Pong {
    start: Instant,
    wait_secs: u64,
}

impl Pong {
    fn new(wait_max_secs: u64) -> Pong {
        let mut rng = rand::thread_rng();
        let wait_secs = rng.gen_range(0, wait_max_secs);
        println!("wait_secs:{}",wait_secs);
        let pong = Pong {
            start: Instant::now(),
            wait_secs
        };
        let duration = Duration::from_millis(1000);
        sleep(duration);
        pong
    }
}



impl Future for Pong {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.start.elapsed().as_secs() > self.wait_secs {
            println!("pong received");
            Ok(Async::Ready(()))
        } else {
            println!("waiting for pong...");
            Ok(Async::NotReady)
        }
    }
}


struct Transport;

impl Transport {
    fn send_ping(&self) {

    }

    fn recv_pong(&self) -> impl Future<Item = (), Error = io::Error> {
        // ...
        println!("entering recv_pong");
        Pong::new(1)
    }
}

fn coordinator_task(rx: mpsc::Receiver<Message>)
                    -> impl Future<Item = (), Error = ()>
{
    let transport = Transport;

    rx.for_each(move |pong_tx| {
        let start = Instant::now();

        transport.send_ping();

        transport.recv_pong()
            .map_err(|e|  println!("recv_pong err = {:?}", e))
            .and_then(move |_| {
                let rtt = start.elapsed();
                pong_tx.send(rtt).unwrap();
                Ok(())
            })
    })
}

/// Request an rtt.
fn rtt(tx: mpsc::Sender<Message>)
       -> impl Future<Item = (Duration, mpsc::Sender<Message>), Error = ()>
{
    let (resp_tx, resp_rx) = oneshot::channel();

    tx.send(resp_tx)
        .map_err(|_| ())
        .and_then(|tx| {
            resp_rx.map(|dur| (dur, tx))
                .map_err(|_| ())
        })
}

fn comm() {
    // 启动应用
    tokio::run(lazy(|| {
        // 创建用于与后台任务通信的通道.
        let (tx, rx) = mpsc::channel(1_024);

        // 创建后台任务：
        tokio::spawn(coordinator_task(rx));

        // 创建少量任务，它们向协调者任务请求 RTT 时间。
        for _ in 0..4 {
            let tx = tx.clone();

            tokio::spawn(lazy(|| {
                rtt(tx).and_then(|(dur, _)| {
                    println!("duration = {:?}", dur);
                    Ok(())
                })
            }));
        }

        Ok(())
    }));
}

#[test]
fn backend_test(){
    backend();

}

#[test]
fn comm_test(){
    comm();

}
