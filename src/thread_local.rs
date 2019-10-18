use std::cell::RefCell;
use std::thread;

thread_local! {
    static FOO: RefCell<u32> = RefCell::new(1);
}

fn local_process() {
    FOO.with(|foo| {
        assert_eq!(*foo.borrow(), 1);
        *foo.borrow_mut() = 2;
    });

    // each thread starts out with the initial value of 1
    thread::spawn(move|| {
        FOO.with(|foo| {
            println!("spawn dagta:{}",*foo.borrow());
            *foo.borrow_mut() = 3;
        });
    });

    // we retain our original value of 2 despite the child thread
    FOO.with(|foo| {
        assert_eq!(*foo.borrow(), 2);
    });
}

#[test]
fn local_test() {
    local_process();
}