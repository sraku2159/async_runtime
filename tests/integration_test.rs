use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread;
use std::time::Duration;

use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

#[test]
fn engine_executes_simple_task() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async { 1 + 1 }, None);

    let result = block_on(receiver);
    assert_eq!(result, 2);

    engine.graceful_shutdown();
}

#[test]
fn engine_executes_multiple_tasks() {
    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    let r1 = engine.reserve(async { 10 }, None);
    let r2 = engine.reserve(async { 20 }, None);
    let r3 = engine.reserve(async { 30 }, None);

    assert_eq!(block_on(r1), 10);
    assert_eq!(block_on(r2), 20);
    assert_eq!(block_on(r3), 30);

    engine.graceful_shutdown();
}

#[test]
fn engine_executes_async_computation() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(
        async {
            let a = 5;
            let b = 7;
            a * b
        },
        None,
    );

    let result = block_on(receiver);
    assert_eq!(result, 35);

    engine.graceful_shutdown();
}

#[test]
fn engine_handles_string_results() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async { "Hello, async runtime!".to_string() }, None);

    let result = block_on(receiver);
    assert_eq!(result, "Hello, async runtime!");

    engine.graceful_shutdown();
}

#[test]
fn engine_with_single_worker() {
    let mut engine = Engine::new(1, |receiver| Box::new(Fifo::new(receiver)));

    let r1 = engine.reserve(async { 100 }, None);
    let r2 = engine.reserve(async { 200 }, None);

    assert_eq!(block_on(r1), 100);
    assert_eq!(block_on(r2), 200);

    engine.graceful_shutdown();
}

#[test]
fn engine_with_many_workers() {
    let mut engine = Engine::new(8, |receiver| Box::new(Fifo::new(receiver)));

    let mut receivers = vec![];
    for i in 0..10 {
        let receiver = engine.reserve(async move { i * 2 }, None);
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
    let receiver = engine.reserve(DummyFuture::new(3), None);

    let result = block_on(receiver);
    assert_eq!(result, 4);

    engine.graceful_shutdown();
}

#[test]
fn engine_handles_multiple_pending_futures() {
    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    // 異なる回数のPoll::Pendingを返すFutureを複数実行
    let r1 = engine.reserve(DummyFuture::new(2), None); // 2回Pending後、Ready(3)
    let r2 = engine.reserve(DummyFuture::new(4), None); // 4回Pending後、Ready(5)
    let r3 = engine.reserve(DummyFuture::new(1), None); // 1回Pending後、Ready(2)

    assert_eq!(block_on(r1), 3);
    assert_eq!(block_on(r2), 5);
    assert_eq!(block_on(r3), 2);

    engine.graceful_shutdown();
}

#[test]
fn deadline_scheduler_orders_tasks_by_deadline() {
    use async_runtime::engine::schedule::deadline::DeadLineScheduler;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Waker};
    use std::thread;
    use std::time::Duration;

    // Pending状態を作れるFuture
    struct ControlledFuture {
        ready: Arc<Mutex<bool>>,
        waker: Arc<Mutex<Option<Waker>>>,
        name: &'static str,
        order: Arc<Mutex<Vec<&'static str>>>,
    }

    impl Future for ControlledFuture {
        type Output = &'static str;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let ready = *self.ready.lock().unwrap();
            if ready {
                self.order.lock().unwrap().push(self.name);
                Poll::Ready(self.name)
            } else {
                *self.waker.lock().unwrap() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    let mut engine = Engine::new(1, |receiver| Box::new(DeadLineScheduler::new(receiver)));

    // 実行順序を記録
    let execution_order = Arc::new(Mutex::new(Vec::new()));

    // 各タスクの準備フラグとWaker
    let ready_b = Arc::new(Mutex::new(false));
    let waker_b = Arc::new(Mutex::new(None));
    let ready_c = Arc::new(Mutex::new(false));
    let waker_c = Arc::new(Mutex::new(None));
    let ready_a = Arc::new(Mutex::new(false));
    let waker_a = Arc::new(Mutex::new(None));

    // 登録順: B (deadline=3000), C (deadline=2000), A (deadline=1000)
    let r1 = engine.reserve(
        ControlledFuture {
            ready: ready_b.clone(),
            waker: waker_b.clone(),
            name: "B",
            order: execution_order.clone(),
        },
        Some(3000),
    );

    let r2 = engine.reserve(
        ControlledFuture {
            ready: ready_c.clone(),
            waker: waker_c.clone(),
            name: "C",
            order: execution_order.clone(),
        },
        Some(2000),
    );

    let r3 = engine.reserve(
        ControlledFuture {
            ready: ready_a.clone(),
            waker: waker_a.clone(),
            name: "A",
            order: execution_order.clone(),
        },
        Some(1000),
    );

    // すべてのタスクが登録され、Pending状態になるまで待つ
    thread::sleep(Duration::from_millis(100));

    // 一斉にReadyにする
    *ready_a.lock().unwrap() = true;
    *ready_c.lock().unwrap() = true;
    *ready_b.lock().unwrap() = true;

    // Wakerを呼ぶ
    if let Some(waker) = waker_a.lock().unwrap().take() {
        waker.wake();
    }
    if let Some(waker) = waker_c.lock().unwrap().take() {
        waker.wake();
    }
    if let Some(waker) = waker_b.lock().unwrap().take() {
        waker.wake();
    }

    // 結果を取得
    assert_eq!(block_on(r3), "A");
    assert_eq!(block_on(r2), "C");
    assert_eq!(block_on(r1), "B");

    // 実行順序を確認: A -> C -> B (deadline順)
    let order = execution_order.lock().unwrap();
    assert_eq!(*order, vec!["A", "C", "B"]);

    engine.graceful_shutdown();
}

#[test]
fn deadline_scheduler_handles_same_deadline() {
    use async_runtime::engine::schedule::deadline::DeadLineScheduler;

    let mut engine = Engine::new(1, |receiver| Box::new(DeadLineScheduler::new(receiver)));

    // 同じdeadlineのタスク
    let r1 = engine.reserve(async { 1 }, Some(1000));
    let r2 = engine.reserve(async { 2 }, Some(1000));
    let r3 = engine.reserve(async { 3 }, Some(1000));

    // すべて完了すること
    assert_eq!(block_on(r1), 1);
    assert_eq!(block_on(r2), 2);
    assert_eq!(block_on(r3), 3);

    engine.graceful_shutdown();
}

#[test]
fn deadline_scheduler_with_none_deadline() {
    use async_runtime::engine::schedule::deadline::DeadLineScheduler;

    let mut engine = Engine::new(1, |receiver| Box::new(DeadLineScheduler::new(receiver)));

    // deadline=Noneのタスク
    let r1 = engine.reserve(async { 100 }, None);
    let r2 = engine.reserve(async { 200 }, Some(1000));
    let r3 = engine.reserve(async { 300 }, None);

    // すべて完了すること
    assert_eq!(block_on(r1), 100);
    assert_eq!(block_on(r2), 200);
    assert_eq!(block_on(r3), 300);

    engine.graceful_shutdown();
}
