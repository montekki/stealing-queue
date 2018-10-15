use std::{thread, time};
pub mod threadpool;
pub mod wsqueue1;

#[macro_use]
extern crate log;
use log::{Level, LevelFilter, Metadata, Record};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
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
