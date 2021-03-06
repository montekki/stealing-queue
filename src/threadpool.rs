//! A thread pool with work-stealing workers
//!
//! Implements a pool of workers that receive work with
//! work-stealing queues and can steal work from each other.
//! The pool adjusts the number of currently running workers
//! according to a number of pending tasks. There is always
//! at least one thread running but not more that a configured
//! number.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::{thread, time};
use wsqueue1::*;

type Queues<T> = Arc<RwLock<Vec<Mutex<WsQueue<T>>>>>;

const MAX_PENDING_TASKS: usize = 10;

pub enum Task {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    max_pending_tasks: usize,
    max_workers: usize,
    workers: Vec<Worker>,
    queues: Queues<Task>,
}

pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
    /// Creates a new thread pool
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);
        let arr = Arc::new(RwLock::new(Vec::new()));

        for _ in 0..1 {
            let mut v = arr.write().unwrap();
            v.push(Mutex::new(WsQueue::new()));
        }

        for i in 0..1 {
            let w = Worker::new(i, &arr.clone());
            workers.push(w);
        }

        ThreadPool {
            max_pending_tasks: MAX_PENDING_TASKS,
            max_workers: size,
            workers,
            queues: arr.clone(),
        }
    }

    /// Sends work to the pool
    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let mut len = 0;
        {
            let a = self.queues.read().unwrap();

            for q in a.iter() {
                let mut lock = q.try_lock();

                if let Ok(ref mut mutex) = lock {
                    len += mutex.len();
                }
            }
        }

        if len > self.max_pending_tasks {
            let mut a = self.queues.write().unwrap();

            if a.len() < self.max_workers {
                info!("Too many tasks, spawning a new worker!");
                a.push(Mutex::new(WsQueue::new()));

                let w = Worker::new(a.len() - 1, &self.queues.clone());
                self.workers.push(w);
            }
        }

        {
            let a = self.queues.read().unwrap();
            let mut q = a[0].lock().unwrap();

            q.push(Task::NewJob(job));
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {}
}

/// A worker that can execute tasks
///
/// It loops and tries to receive tasks from it's own
/// queue or to steal tasks from other queues until it's dropped
struct Worker {
    // id is the index of our queue in the Queues vector
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
    queues: Queues<Task>,
    should_stop: Arc<AtomicBool>,
}

impl Worker {
    fn start(&mut self) {
        let should_stop = self.should_stop.clone();
        let queues = self.queues.clone();
        let id = self.id;

        let thread = thread::spawn(move || {
            while !should_stop.load(Ordering::SeqCst) {
                let qs = queues.read().unwrap();

                debug!("Thread {} qs.len {}", id, qs.len());

                if qs.len() > id {
                    let mut work;

                    {
                        let mut myqueue = qs[id].lock().unwrap();
                        work = myqueue.pop();
                    }
                    if work.is_none() {
                        debug!("Nothing is on the local queue for thread {}", id);

                        for (i, _) in qs.iter().enumerate() {
                            if i == id {
                                continue;
                            }
                            {
                                let mut lock = qs[i].try_lock();
                                if let Ok(ref mut mutex) = lock {
                                    work = mutex.pop();
                                } else {
                                    continue;
                                }
                            }
                            if work.is_some() {
                                debug!("Have managed to steal work from queue {}!", i);
                                break;
                            }
                        }
                    }
                    match work {
                        None => {
                            debug!("Could not steal from the other queues");
                            thread::sleep(time::Duration::new(1, 0));
                        }
                        Some(t) => {
                            debug!("Got some work!");
                            match t {
                                Task::Terminate => {
                                    debug!("Terminating worker {}", id);
                                }
                                Task::NewJob(j) => {
                                    debug!("Got new job in task {}", id);
                                    j.call_box();
                                }
                            }
                        }
                    }
                }
            }
        });

        self.thread = Some(thread);
    }

    /// Creates a new worker
    pub fn new(id: usize, q: &Arc<RwLock<Vec<Mutex<WsQueue<Task>>>>>) -> Worker {
        let mut w = Worker {
            id,
            thread: None,
            queues: Arc::clone(&q),
            should_stop: Arc::new(AtomicBool::new(false)),
        };
        w.start();
        w
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.should_stop.store(true, Ordering::SeqCst);
            thread.join().unwrap();
            info!("Stopped worker {}", self.id);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Worker;
    use super::*;
    use std::sync::{Arc, Mutex, RwLock};

    #[test]
    fn worker_basic_test() {
        let queuenum = 4;
        let arr = Arc::new(RwLock::new(Vec::new()));

        for _ in 0..queuenum {
            let mut v = arr.write().unwrap();
            v.push(Mutex::new(WsQueue::new()));
        }

        let _w = Worker::new(0, &arr.clone());

        for i in 0..queuenum {
            let a = arr.read().unwrap();
            let mut q = a[i].lock().unwrap();

            q.push(Task::NewJob(Box::new(move || {
                println!("new job {}", i);
            })));
        }

        thread::sleep(time::Duration::new(10, 0));

        for i in 0..queuenum {
            let a = arr.read().unwrap();
            let mut q = a[i].lock().unwrap();

            let res = q.pop();
            match res {
                None => (),
                Some(_) => panic!("The queues have not been emptied by the worker"),
            }
        }
    }

    #[test]
    fn pool_basic_test() {
        let mut pool = ThreadPool::new(4);

        pool.execute(|| {
            println!("task1");
            thread::sleep(time::Duration::new(5, 0));
        });
        pool.execute(|| {
            println!("task2");
        });

        thread::sleep(time::Duration::new(10, 0));
    }
}
