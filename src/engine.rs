pub mod schedule;
pub mod task;
pub mod waker;
pub mod worker;

use schedule::Scheduler;
use std::{
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::channel as mpsc_channel,
    },
    task::Wake,
    thread::{JoinHandle, spawn},
};
use worker::{Worker, WorkerInfo};

use task::Task;

use crate::utils::channel::{Receiver, channel};

pub struct Engine {
    scheduler: Arc<Mutex<Box<dyn Scheduler + Send + 'static>>>,
    shutdown: Arc<AtomicBool>,
    worker_threads: Vec<JoinHandle<()>>,
    worker_handles: std::sync::mpsc::Receiver<std::thread::Thread>,
}

impl Engine {
    pub fn new(
        worker_num: usize,
        scheduler_factory: impl FnOnce(
            std::sync::mpsc::Receiver<WorkerInfo>,
        ) -> Box<dyn Scheduler + Send + 'static>,
    ) -> Self {
        let (worker_sender, worker_receiver) = mpsc_channel();
        let (handle_sender, handle_receiver) = mpsc_channel();
        let scheduler = Arc::new(Mutex::new(scheduler_factory(worker_receiver)));
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut worker_threads = Vec::new();
        for _ in 0..worker_num {
            let cloned_scheduler = scheduler.clone();
            let cloned_sender = worker_sender.clone();
            let cloned_shutdown = shutdown.clone();
            let cloned_handle_sender = handle_sender.clone();

            let tj = spawn(move || {
                // スレッドハンドルを送信
                let _ = cloned_handle_sender.send(std::thread::current());
                Worker::new(cloned_sender, cloned_scheduler, cloned_shutdown).execute();
            });

            worker_threads.push(tj);
        }
        Self {
            scheduler,
            shutdown,
            worker_threads,
            worker_handles: handle_receiver,
        }
    }

    pub fn reserve<V, W>(&mut self, task: V) -> Receiver<W>
    where
        V: Future<Output = W> + Send + 'static,
        W: Clone + Send + Unpin + 'static,
    {
        let (sender, receiver) = channel();
        let task = Task::new(task, sender);
        self.scheduler.lock().unwrap().schedule(task);
        receiver
    }

    pub fn graceful_shutdown(self) {
        self.shutdown.store(true, Ordering::Release);

        while let Ok(thread) = self.worker_handles.try_recv() {
            thread.unpark();
        }

        for tj in self.worker_threads {
            let _ = tj.join();
        }
    }
}

pub fn block_on<T, F: IntoFuture<Output = T>>(future: F) -> T {
    use std::{
        sync::Arc,
        task::{Context, Poll},
        thread,
    };

    struct Waker {
        t: thread::Thread,
    }

    impl Wake for Waker {
        fn wake(self: std::sync::Arc<Self>) {
            eprintln!("waker in block_on is called");
            self.t.unpark();
        }
    }

    let waker = std::task::Waker::from(Arc::new(Waker {
        t: thread::current(),
    }));
    let mut cx = Context::from_waker(&waker);
    let mut future = Box::pin(future.into_future());

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => thread::park(),
        }
    }
}

#[cfg(test)]
mod test;
