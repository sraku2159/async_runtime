use crate::engine::schedule::Scheduler;

use super::Fifo;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

#[derive(PartialEq, Eq)]
struct DummyTask {}

impl Future for DummyTask {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Ready(())
    }
}

#[test]
fn test_take_task() {
    let mut scheduler = Fifo::new();

    let task1: Pin<Box<dyn Future<Output = ()>>> = Box::pin(DummyTask {});
    let task2: Pin<Box<dyn Future<Output = ()>>> = Box::pin(DummyTask {});

    let task_ptr = task1.as_ref().get_ref() as *const _ as *const ();

    scheduler.schedule(task1);
    scheduler.schedule(task2);
    let retrieved_task = scheduler.take().expect("Task should be present");

    let retrieved_ptr = retrieved_task.as_ref().get_ref() as *const _ as *const ();
    assert_eq!(task_ptr, retrieved_ptr, "Pointers should match");
}
