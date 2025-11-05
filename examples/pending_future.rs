use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

/// Poll::Pendingを返すカスタムFuture
/// cnt が max_polls に達するまで Poll::Pending を返し続ける
/// （正しい実装：別スレッドからWakerを呼ぶ）
struct CountingFuture {
    cnt: Arc<Mutex<usize>>,
    max_polls: usize,
    value: i32,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl CountingFuture {
    fn new(max_polls: usize, value: i32) -> Self {
        println!(
            "Creating CountingFuture: will poll {} times before returning {}",
            max_polls, value
        );
        Self {
            cnt: Arc::new(Mutex::new(0)),
            max_polls,
            value,
            waker: Arc::new(Mutex::new(None)),
        }
    }
}

impl Future for CountingFuture {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut cnt = self.cnt.lock().unwrap();
        *cnt += 1;
        let current_cnt = *cnt;
        println!(
            "  CountingFuture poll attempt #{} (max: {})",
            current_cnt, self.max_polls
        );

        if current_cnt >= self.max_polls {
            println!("  -> Ready! Returning {}", self.value);
            Poll::Ready(self.value)
        } else {
            println!("  -> Pending... will be woken up asynchronously");
            // Wakerを保存
            *self.waker.lock().unwrap() = Some(cx.waker().clone());

            // 別スレッドで少し待ってからWakerを呼ぶ（正しいパターン）
            let waker_clone = Arc::clone(&self.waker);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(10));
                if let Some(waker) = waker_clone.lock().unwrap().take() {
                    waker.wake();
                }
            });

            Poll::Pending
        }
    }
}

fn main() {
    println!("=== Pending Future Example ===\n");

    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    println!("Submitting task 1: CountingFuture(3 polls, value=100)");
    let r1 = engine.reserve(CountingFuture::new(3, 100), None);

    println!("\nSubmitting task 2: CountingFuture(5 polls, value=200)");
    let r2 = engine.reserve(CountingFuture::new(5, 200), None);

    println!("\nSubmitting task 3: CountingFuture(2 polls, value=300)");
    let r3 = engine.reserve(CountingFuture::new(2, 300), None);

    println!("\n--- Waiting for results ---\n");

    let result1 = block_on(r1);
    println!("Task 1 result: {}", result1);

    let result2 = block_on(r2);
    println!("Task 2 result: {}", result2);

    let result3 = block_on(r3);
    println!("Task 3 result: {}", result3);

    println!("\n=== All tasks completed successfully! ===");

    engine.graceful_shutdown();
}
