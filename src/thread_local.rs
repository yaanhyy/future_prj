use std::cell::RefCell;
use std::thread;
use core::borrow::BorrowMut;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);
thread_local! {
    static FOO: usize = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
}

fn local_process() {
    FOO.with(|mut foo| {
        assert_eq!(*foo, 1);
    });

    // each thread starts out with the initial value of 1
    thread::spawn(move|| {
        FOO.with(|& foo| {
            println!("spawn dagta:{}", foo);

        });
    });

    // each thread starts out with the initial value of 1
    thread::spawn(move|| {
        FOO.with(|& foo| {
            println!("spawn dagta:{}", foo);

        });
    });
    // we retain our original value of 2 despite the child thread
    FOO.with(|mut foo| {
        assert_eq!(*foo, 1);
    });
}

#[test]
fn local_test() {
    local_process();
}