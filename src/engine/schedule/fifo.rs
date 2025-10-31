use crate::engine::task::Task;

use super::Scheduler;

use std::collections::VecDeque;
use std::pin::Pin;

pub struct Fifo {
    queue: VecDeque<Pin<Box<dyn Future<Output = ()> + 'static>>>,
}

impl Fifo {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl Scheduler for Fifo {
    fn schedule<T>(&mut self, task: Pin<Box<dyn Future<Output = T>>>)
    where
        T: Unpin + 'static,
    {
        let task = Box::pin(Task::<T>::new(task));
        self.queue.push_back(task);
    }

    fn take(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod test;
