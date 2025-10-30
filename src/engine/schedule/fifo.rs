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
    fn schedule(&mut self, task: Pin<Box<dyn Future<Output = ()>>>) {
        self.queue.push_back(task);
    }

    fn take(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod test;
