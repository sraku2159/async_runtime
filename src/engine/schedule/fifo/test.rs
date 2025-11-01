use crate::engine::schedule::Scheduler;
use crate::engine::task::{Task, TaskTrait};
use crate::utils::channel::channel;

use super::Fifo;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

#[derive(PartialEq, Eq)]
struct DummyTask {}

impl Future for DummyTask {
    type Output = i32;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Ready(42)
    }
}

#[test]
fn test_take_task() {
    let (sender, _) = channel();
    let task1: Pin<Box<dyn TaskTrait>> = Box::pin(Task::new(DummyTask {}, sender));
    let (sender, _) = channel();
    let task2: Pin<Box<dyn TaskTrait>> = Box::pin(Task::new(DummyTask {}, sender));

    let mut scheduler = Fifo::new();

    let task_ptr = task1.as_ref().get_ref() as *const _ as *const ();

    scheduler.schedule(task1);
    scheduler.schedule(task2);
    let retrieved_task = scheduler.take().expect("Task should be present");

    let retrieved_ptr = retrieved_task.as_ref().get_ref() as *const _ as *const ();
    assert_eq!(task_ptr, retrieved_ptr, "Pointers should match");
}
