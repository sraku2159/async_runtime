use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

/// Wakerを別スレッドから非同期的に呼ぶFuture
struct AsyncWakerFuture {
    waker_sent: Arc<Mutex<bool>>,
    waker: Arc<Mutex<Option<Waker>>>,
    value: i32,
}

impl AsyncWakerFuture {
    fn new(delay_ms: u64, value: i32) -> Self {
        println!(
            "Creating AsyncWakerFuture: will wake after {}ms with value {}",
            delay_ms, value
        );
        let waker_sent = Arc::new(Mutex::new(false));
        let waker = Arc::new(Mutex::new(None));

        let waker_sent_clone = Arc::clone(&waker_sent);
        let waker_clone = Arc::clone(&waker);

        // 別スレッドで遅延後にWakerを呼ぶ
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(delay_ms));
            let waker_opt: Option<Waker> = waker_clone.lock().unwrap().take();
            if let Some(waker) = waker_opt {
                println!(
                    "  [Background thread] Waking task after {}ms delay",
                    delay_ms
                );
                waker.wake();
                *waker_sent_clone.lock().unwrap() = true;
            }
        });

        Self {
            waker_sent,
            waker,
            value,
        }
    }
}

impl Future for AsyncWakerFuture {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker_sent = self.waker_sent.lock().unwrap();

        if *waker_sent {
            println!(
                "  AsyncWakerFuture: Waker was called, returning Ready({})",
                self.value
            );
            Poll::Ready(self.value)
        } else {
            println!("  AsyncWakerFuture: Not ready yet, storing waker and returning Pending");
            // Wakerを保存（別スレッドが呼び出すため）
            *self.waker.lock().unwrap() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

fn main() {
    println!("=== Waker Test Example ===\n");
    println!("This example demonstrates async tasks that are woken by background threads\n");

    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    println!("Submitting task 1: AsyncWakerFuture(100ms delay, value=100)");
    let r1 = engine.reserve(AsyncWakerFuture::new(100, 100), None);

    println!("\nSubmitting task 2: AsyncWakerFuture(200ms delay, value=200)");
    let r2 = engine.reserve(AsyncWakerFuture::new(200, 200), None);

    println!("\nSubmitting task 3: AsyncWakerFuture(50ms delay, value=300)");
    let r3 = engine.reserve(AsyncWakerFuture::new(50, 300), None);

    println!("\n--- Waiting for results ---\n");

    let result3 = block_on(r3);
    println!("Task 3 result: {}", result3);

    let result1 = block_on(r1);
    println!("Task 1 result: {}", result1);

    let result2 = block_on(r2);
    println!("Task 2 result: {}", result2);

    println!("\n=== All tasks completed! ===");

    engine.graceful_shutdown();
}
