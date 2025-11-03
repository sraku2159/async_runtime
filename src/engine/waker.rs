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
        // PENDING状態のタスクのみ再スケジュール
        // poll()が終了してPENDINGになった後に、外部イベントからwakeが呼ばれる想定
        if self.task.get_state() == task::PENDING {
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
