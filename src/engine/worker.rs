use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::task::{self, Context};
use std::thread;

use crate::engine::schedule::Scheduler;
use crate::engine::task::SharedTask;
use crate::engine::waker;

pub struct Worker {
    worker_sender: Sender<WorkerInfo>,
    t_sender: Sender<SharedTask>,
    t_receiver: Receiver<SharedTask>,
    scheduler: Arc<Mutex<Box<dyn Scheduler + Send>>>,
    shutdown: Arc<AtomicBool>,
}

impl Worker {
    pub fn new(
        worker_sender: Sender<WorkerInfo>,
        scheduler: Arc<Mutex<Box<dyn Scheduler + Send>>>,
        shutdown: Arc<AtomicBool>,
    ) -> Self {
        let (t_sender, t_receiver) = channel();
        Self {
            worker_sender,
            t_receiver,
            t_sender,
            scheduler,
            shutdown,
        }
    }

    pub fn execute(&self) {
        loop {
            if self.shutdown.load(Ordering::Acquire) {
                break;
            }

            let _ = self.worker_sender.send(WorkerInfo {
                t: thread::current(),
                sender: self.t_sender.clone(),
            });

            self.scheduler.lock().unwrap().notify();

            thread::park();

            if self.shutdown.load(Ordering::Acquire) {
                break;
            }

            if let Ok(task) = self.t_receiver.recv() {
                let waker = waker::Waker::new(self.scheduler.clone(), Arc::clone(&task));
                let waker = task::Waker::from(Arc::new(waker));
                let mut context = Context::from_waker(&waker);
                let _ = task.poll(&mut context);
            }
        }
    }
}

pub struct WorkerInfo {
    pub t: thread::Thread,
    pub sender: Sender<SharedTask>,
}

#[cfg(test)]
mod test;
