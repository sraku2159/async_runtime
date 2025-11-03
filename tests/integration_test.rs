use std::task::{Poll, Waker};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

#[test]
fn engine_executes_simple_task() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async { 1 + 1 });

    let result = block_on(receiver);
    assert_eq!(result, 2);

    engine.graceful_shutdown();
}

#[test]
fn engine_executes_multiple_tasks() {
    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    let r1 = engine.reserve(async { 10 });
    let r2 = engine.reserve(async { 20 });
    let r3 = engine.reserve(async { 30 });

    assert_eq!(block_on(r1), 10);
    assert_eq!(block_on(r2), 20);
    assert_eq!(block_on(r3), 30);

    engine.graceful_shutdown();
}

#[test]
fn engine_executes_async_computation() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async {
        let a = 5;
        let b = 7;
        a * b
    });

    let result = block_on(receiver);
    assert_eq!(result, 35);

    engine.graceful_shutdown();
}

#[test]
fn engine_handles_string_results() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async { "Hello, async runtime!".to_string() });

    let result = block_on(receiver);
    assert_eq!(result, "Hello, async runtime!");

    engine.graceful_shutdown();
}

#[test]
fn engine_with_single_worker() {
    let mut engine = Engine::new(1, |receiver| Box::new(Fifo::new(receiver)));

    let r1 = engine.reserve(async { 100 });
    let r2 = engine.reserve(async { 200 });

    assert_eq!(block_on(r1), 100);
    assert_eq!(block_on(r2), 200);

    engine.graceful_shutdown();
}

#[test]
fn engine_with_many_workers() {
    let mut engine = Engine::new(8, |receiver| Box::new(Fifo::new(receiver)));

    let mut receivers = vec![];
    for i in 0..10 {
        let receiver = engine.reserve(async move { i * 2 });
        receivers.push(receiver);
    }

    for (i, receiver) in receivers.into_iter().enumerate() {
        assert_eq!(block_on(receiver), i * 2);
    }

    engine.graceful_shutdown();
}

struct DummyFuture {
    num: i32,
    cnt: Arc<Mutex<i32>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl DummyFuture {
    fn new(num: i32) -> Self {
        Self {
            num,
            cnt: Arc::new(Mutex::new(0)),
            waker: Arc::new(Mutex::new(None)),
        }
    }
}

impl std::future::Future for DummyFuture {
    type Output = i32;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut cnt = self.cnt.lock().unwrap();

        if *cnt > self.num {
            Poll::Ready(*cnt)
        } else {
            *cnt += 1;
            // Wakerを保存
            *self.waker.lock().unwrap() = Some(cx.waker().clone());

            // 別スレッドで少し待ってからWakerを呼ぶ（正しいパターン）
            let waker_clone = Arc::clone(&self.waker);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(5));
                if let Some(waker) = waker_clone.lock().unwrap().take() {
                    waker.wake();
                }
            });

            Poll::Pending
        }
    }
}

#[test]
fn engine_handles_pending_future() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    // DummyFuture::new(3) は cnt > 3 になるまでPoll::Pendingを返す
    // poll: cnt=0->1 (Pending), cnt=1->2 (Pending), cnt=2->3 (Pending), cnt=3->4 (Ready(4))
    let receiver = engine.reserve(DummyFuture::new(3));

    let result = block_on(receiver);
    assert_eq!(result, 4);

    engine.graceful_shutdown();
}

#[test]
fn engine_handles_multiple_pending_futures() {
    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    // 異なる回数のPoll::Pendingを返すFutureを複数実行
    let r1 = engine.reserve(DummyFuture::new(2)); // 2回Pending後、Ready(3)
    let r2 = engine.reserve(DummyFuture::new(4)); // 4回Pending後、Ready(5)
    let r3 = engine.reserve(DummyFuture::new(1)); // 1回Pending後、Ready(2)

    assert_eq!(block_on(r1), 3);
    assert_eq!(block_on(r2), 5);
    assert_eq!(block_on(r3), 2);

    engine.graceful_shutdown();
}
