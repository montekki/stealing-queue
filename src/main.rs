use std::{thread, time};

pub mod threadpool;
pub mod wsqueue1;

fn main() {
    let mut pool = threadpool::ThreadPool::new(4);

    for i in 0..20 {
        pool.execute(move || {
            println!("TASK {}", i);
            thread::sleep(time::Duration::new(1, 0));
            println!("TASK {} END", i);
        });
        if i % 5 == 0 {
            thread::sleep(time::Duration::new(1, 0));
        }
    }

    pool.execute(|| {
        println!("TASK2");
    });

    thread::sleep(time::Duration::new(30, 0));
}
