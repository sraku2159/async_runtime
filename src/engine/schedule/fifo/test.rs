use crate::engine::schedule::Scheduler;
use crate::engine::task::Task;
use crate::engine::worker::WorkerInfo;
use crate::utils::channel::channel;

use super::Fifo;
use std::future::Future;
use std::sync::Arc;
use std::task::Poll;
use std::thread;

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
fn take_task_one_by_one() {
    let (sender, _) = channel();
    let task1 = Task::new(DummyTask {}, sender, None);
    let (sender, _) = channel();
    let task2 = Task::new(DummyTask {}, sender, None);

    let (worker_sender, worker_receiver) = std::sync::mpsc::channel();
    let mut scheduler = Fifo::new(worker_receiver);

    let task1_ptr = Arc::as_ptr(&task1);
    let task2_ptr = Arc::as_ptr(&task2);

    // Schedule tasks
    scheduler.schedule(task1);
    scheduler.schedule(task2);

    // Create channels to receive tasks from the scheduler
    let (task_sender1, task_receiver1) = std::sync::mpsc::channel();
    let (task_sender2, task_receiver2) = std::sync::mpsc::channel();

    // Send worker info and verify first task is retrieved
    worker_sender
        .send(WorkerInfo {
            t: thread::current(),
            sender: task_sender1,
        })
        .unwrap();

    // Send second worker info
    worker_sender
        .send(WorkerInfo {
            t: thread::current(),
            sender: task_sender2,
        })
        .unwrap();

    // Trigger notification
    scheduler.notify();

    // Verify tasks are received in FIFO order
    let retrieved_task = task_receiver1.recv().expect("Task should be present");
    let retrieved_ptr = Arc::as_ptr(&retrieved_task);
    assert_eq!(task1_ptr, retrieved_ptr, "First task should match task1");

    let retrieved_task = task_receiver2.recv().expect("Task should be present");
    let retrieved_ptr = Arc::as_ptr(&retrieved_task);
    assert_eq!(task2_ptr, retrieved_ptr, "Second task should match task2");
}
