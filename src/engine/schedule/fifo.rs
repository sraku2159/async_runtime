use crate::engine::task::{self, TaskTrait};

use super::Scheduler;

use std::collections::VecDeque;
use std::pin::Pin;

pub struct Fifo {
    queue: VecDeque<Pin<Box<dyn TaskTrait<Output = ()> + 'static>>>,
}

impl Fifo {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl Scheduler for Fifo {
    fn schedule(&mut self, mut task: Pin<Box<dyn TaskTrait>>) {
        task.as_mut().set_state(task::SCHEDULED);
        self.queue.push_back(task);
    }

    fn take(&mut self) -> Option<Pin<Box<dyn TaskTrait>>> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod test;
