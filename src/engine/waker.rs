use std::sync::{Arc, Mutex};
use std::task::Wake;

use crate::engine::schedule::Scheduler;
use crate::engine::task::{self, SharedTask};

pub struct Waker<T>
where
    T: Scheduler,
{
    scheduler: Arc<Mutex<T>>,
    task: SharedTask,
}

impl<T> Waker<T>
where
    T: Scheduler,
{
    pub fn new(schedule: Arc<Mutex<T>>, task: SharedTask) -> Self {
        Self {
            scheduler: schedule,
            task,
        }
    }
}

impl<T> Wake for Waker<T>
where
    T: Scheduler,
{
    fn wake(self: Arc<Self>) {
        eprintln!("called waker");
        let state = self.task.get_state();
        // RUNNING状態でもwakeを許可（poll()内でwake_by_ref()が呼ばれた場合）
        // PENDING状態でもwakeを許可（poll()が既に終わった後にwakeが呼ばれた場合）
        if state == task::PENDING || state == task::RUNNING {
            self.scheduler
                .lock()
                .unwrap()
                .schedule(Arc::clone(&self.task));
            eprintln!("task reshceduled!!");
        }
    }
}

#[cfg(test)]
mod test;
