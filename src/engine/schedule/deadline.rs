use std::fmt::Debug;

use crate::engine::task::Task;

pub struct DeadLineScheduler {}
// TODO: どうやってデッドラインを設定するか考える

struct Heap<T: PartialOrd>(Vec<T>);

impl<T> Heap<T>
where
    T: PartialOrd + Debug,
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

struct TaskWithDeadLine {
    task: Task,
    deadline: u64,
}

impl TaskWithDeadLine {
    pub fn new(task: Task, deadline: u64) -> Self {
        Self {
            task: task,
            deadline: deadline,
        }
    }
}

impl PartialEq for TaskWithDeadLine {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl PartialOrd for TaskWithDeadLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deadline.partial_cmp(&other.deadline)
    }
}

#[cfg(test)]
mod test;
