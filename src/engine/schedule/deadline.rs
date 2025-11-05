use std::{collections::VecDeque, sync::mpsc::Receiver};

use crate::engine::{schedule::Scheduler, task::SharedTask, worker::WorkerInfo};

pub struct DeadLineScheduler {
    receiver: Receiver<WorkerInfo>,
    pending_workers: VecDeque<WorkerInfo>,
    heap: Heap<SharedTask>,
}

impl DeadLineScheduler {
    pub fn new(receiver: Receiver<WorkerInfo>) -> Self {
        Self {
            receiver,
            pending_workers: VecDeque::new(),
            heap: Heap::new(),
        }
    }
}

impl Scheduler for DeadLineScheduler {
    fn register(&mut self, task: SharedTask) {
        self.heap.insert(task);
    }

    fn take(&mut self) -> Option<SharedTask> {
        self.heap.delete()
    }

    fn get_pending_workers(&mut self) -> &mut VecDeque<WorkerInfo> {
        &mut self.pending_workers
    }

    fn get_worker_receiver(&mut self) -> &mut Receiver<WorkerInfo> {
        &mut self.receiver
    }
}

struct Heap<T: PartialOrd>(Vec<T>);

impl<T> Heap<T>
where
    T: PartialOrd,
{
    fn new() -> Self {
        Heap(Vec::new())
    }

    fn insert(&mut self, task: T) {
        let values = &mut self.0;
        values.push(task);
        let mut idx = values.len() - 1;
        if idx == 0 {
            return;
        }
        let mut parent = (idx - 1) / 2;
        while values[idx] < values[parent] {
            values.swap(idx, parent);
            if parent == 0 {
                return;
            }
            idx = parent;
            parent = (parent - 1) / 2;
        }
    }

    fn delete(&mut self) -> Option<T> {
        let len = self.0.len();
        let values = &mut self.0;
        if len == 0 {
            return None;
        }
        values.swap(0, len - 1);
        let ret = self.0.pop();
        self.normalize();
        ret
    }

    fn normalize(&mut self) {
        while (self.0.len() > 1 && self.0[1] < self.0[0])
            || (self.0.len() > 2 && self.0[2] < self.0[0])
        {
            self.inner_normalize();
        }
    }

    fn inner_normalize(&mut self) {
        let mut idx = 0;
        let mut child_base = idx * 2;

        while let Some(offset) = (1..=2)
            .into_iter()
            .filter(|x| (child_base + x) < self.0.len())
            .min_by(|&a, &b| {
                self.0[child_base + a]
                    .partial_cmp(&self.0[child_base + b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|&x| self.0[child_base + x] < self.0[idx])
        {
            let child = child_base + offset;
            self.0.swap(idx, child);
            idx = child;
            child_base = child * 2;
        }
    }
}

#[cfg(test)]
mod test;
