use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Wake;

use crate::engine::schedule::Scheduler;
use crate::engine::task::{self, TaskTrait};

pub struct Waker<T>
where
    T: Scheduler,
{
    scheduler: Arc<Mutex<T>>,
    task: Mutex<Option<Pin<Box<dyn TaskTrait>>>>,
}

impl<T> Waker<T>
where
    T: Scheduler,
{
    pub fn new(schedule: Arc<Mutex<T>>, task: Pin<Box<dyn TaskTrait>>) -> Self {
        Self {
            scheduler: schedule,
            task: Mutex::new(Some(task)),
        }
    }
}

impl<T> Wake for Waker<T>
where
    T: Scheduler,
{
    fn wake(self: Arc<Self>) {
        if let Some(task) = self.task.lock().unwrap().take()
            && task.as_ref().get_state() == task::PENDING
        {
            self.scheduler.lock().unwrap().schedule(task);
        }
    }
}

#[cfg(test)]
mod test;
