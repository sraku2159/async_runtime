use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::{Poll, Wake},
};

use crate::engine::{
    schedule::Scheduler,
    task::{self, TaskTrait},
    waker::Waker,
};

struct DummyTask {
    state: u8,
}

impl DummyTask {
    fn new(state: u8) -> Self {
        Self { state }
    }
}

impl TaskTrait for DummyTask {
    fn set_state(mut self: std::pin::Pin<&mut Self>, val: u8) {
        self.state = val;
    }

    fn get_state(self: std::pin::Pin<&Self>) -> u8 {
        self.state
    }
}

impl Future for DummyTask {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Ready(())
    }
}

struct DummyScheduler {
    scheduled_count: usize,
    tasks: Vec<std::pin::Pin<Box<dyn TaskTrait>>>,
}

impl DummyScheduler {
    fn new() -> Self {
        Self {
            scheduled_count: 0,
            tasks: Vec::new(),
        }
    }
}

impl Scheduler for DummyScheduler {
    fn schedule(&mut self, task: std::pin::Pin<Box<dyn TaskTrait>>) {
        self.scheduled_count += 1;
        self.tasks.push(task);
    }

    fn take(&mut self) -> Option<std::pin::Pin<Box<dyn TaskTrait>>> {
        self.tasks.pop()
    }
}

fn test_waker_schedule_count(task_state: u8, expected_count: usize) {
    let scheduler = Arc::new(Mutex::new(DummyScheduler::new()));
    let task = Box::pin(DummyTask::new(task_state));
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
