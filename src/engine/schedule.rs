pub mod fifo;

use std::pin::Pin;

use crate::engine::task::TaskTrait;

pub trait Scheduler {
    // アルゴリズムは自由だが、task.set_state(Task::SCHEDULED)は呼ばないといけない
    fn schedule(&mut self, task: Pin<Box<dyn TaskTrait>>);
    fn take(&mut self) -> Option<Pin<Box<dyn TaskTrait>>>;
}
