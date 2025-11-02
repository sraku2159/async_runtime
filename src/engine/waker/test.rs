use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::{Poll, Wake},
};

use crate::engine::{
    schedule::Scheduler,
    task::{self, SharedTask, Task},
    waker::Waker,
};

struct DummyFuture {}

impl Future for DummyFuture {
    type Output = i32;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Ready(42)
    }
}

struct DummyScheduler {
    scheduled_count: usize,
    tasks: Vec<SharedTask>,
    worker_receiver: std::sync::mpsc::Receiver<crate::engine::worker::WorkerInfo>,
    pending_workers: std::collections::VecDeque<crate::engine::worker::WorkerInfo>,
}
impl DummyScheduler {
    fn new() -> Self {
        let (_, worker_receiver) = std::sync::mpsc::channel();
        Self {
            scheduled_count: 0,
            tasks: Vec::new(),
            worker_receiver,
            pending_workers: std::collections::VecDeque::new(),
        }
    }
}

impl Scheduler for DummyScheduler {
    fn register(&mut self, task: SharedTask) {
        self.scheduled_count += 1;
        self.tasks.push(task);
    }

    fn take(&mut self) -> Option<SharedTask> {
        self.tasks.pop()
    }

    fn get_pending_workers(&mut self) -> &mut std::collections::VecDeque<crate::engine::worker::WorkerInfo> {
        &mut self.pending_workers
    }

    fn get_worker_receiver(&mut self) -> &mut std::sync::mpsc::Receiver<crate::engine::worker::WorkerInfo> {
        &mut self.worker_receiver
    }
}

fn test_waker_schedule_count(task_state: u8, expected_count: usize) {
    let scheduler = Arc::new(Mutex::new(DummyScheduler::new()));
    let (sender, _) = crate::utils::channel::channel();
    let task = Task::new(DummyFuture {}, sender);
    task.set_state(task_state);
    let waker = Arc::new(Waker::new(scheduler.clone(), task));

    waker.wake();

    assert_eq!(scheduler.lock().unwrap().scheduled_count, expected_count);
}

#[test]
fn not_schedule_already_scheduled() {
    test_waker_schedule_count(task::SCHEDULED, 0);
}

#[test]
fn not_schedule_running() {
    test_waker_schedule_count(task::RUNNING, 0);
}

#[test]
fn schedule_pending() {
    test_waker_schedule_count(task::PENDING, 1);
}

#[test]
fn schedule_completed() {
    test_waker_schedule_count(task::COMPLETED, 0);
}
