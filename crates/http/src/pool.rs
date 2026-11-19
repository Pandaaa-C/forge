use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    sender: Option<Sender<Job>>,
    workers: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "thread pool needs at least one worker");
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| {
                let receiver = Arc::clone(&receiver);
                thread::Builder::new()
                    .name(format!("forge-worker-{id}"))
                    .spawn(move || worker_loop(receiver))
                    .expect("failed to spawn worker thread")
            })
            .collect();

        Self { sender: Some(sender), workers }
    }

    pub fn execute(&self, job: impl FnOnce() + Send + 'static) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Box::new(job));
        }
    }
}

fn worker_loop(receiver: Arc<Mutex<Receiver<Job>>>) {
    loop {
        let job = match receiver.lock() {
            Ok(rx) => rx.recv(),
            Err(poisoned) => poisoned.into_inner().recv(),
        };
        match job {
            Ok(job) => {
                let _ = catch_unwind(AssertUnwindSafe(job));
            }
            Err(_) => break,
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take()); 
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
