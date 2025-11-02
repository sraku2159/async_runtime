use crate::engine::task::SharedTask;
use crate::engine::worker::WorkerInfo;

use super::Scheduler;

use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

pub struct Fifo {
    queue: VecDeque<SharedTask>,
    worker_receiver: Receiver<WorkerInfo>,
    pending_workers: VecDeque<WorkerInfo>,
}

impl Fifo {
    pub fn new(worker_receiver: Receiver<WorkerInfo>) -> Self {
        Self {
            queue: VecDeque::new(),
            worker_receiver,
            pending_workers: VecDeque::new(),
        }
    }
}

impl Scheduler for Fifo {
    fn register(&mut self, task: SharedTask) {
        self.queue.push_back(task);
    }

    fn take(&mut self) -> Option<SharedTask> {
        self.queue.pop_front()
    }

    fn get_pending_workers(&mut self) -> &mut VecDeque<WorkerInfo> {
        &mut self.pending_workers
    }

    fn get_worker_receiver(&mut self) -> &mut Receiver<WorkerInfo> {
        &mut self.worker_receiver
    }
}

#[cfg(test)]
mod test;
