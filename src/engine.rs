pub mod schedule;
pub mod task;
pub mod waker;
pub mod worker;

use schedule::Scheduler;
use std::{
    future::{self, Future},
    task::Wake,
};
use worker::Worker;

use task::Task;

use crate::utils::channel::{Receiver, channel};

pub struct Engine<T, U>
where
    T: Scheduler,
    U: Worker,
{
    scheduler: T,
    worker: U,
}

impl<T, U> Engine<T, U>
where
    T: Scheduler,
    U: Worker,
{
    // workerのインスタンスかがされた時点で、別スレッドで起動してる
    pub fn new(scheduler: T, worker: U) -> Self {
        Self { scheduler, worker }
    }

    pub fn reserve<V, W>(&mut self, task: V) -> Receiver<W>
    where
        V: Future<Output = W> + 'static,
        W: Clone + Unpin + 'static,
    {
        let (sender, receiver) = channel();
        let task = Box::pin(Task::new(Box::pin(task), sender));
        self.scheduler.schedule(task);
        receiver
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
