use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

/// 非ブロッキングSleep Future（別スレッドでsleep）
struct NonBlockingSleep {
    duration: Duration,
    started_at: Option<Instant>,
    waker_sent: Arc<Mutex<bool>>,
}

impl NonBlockingSleep {
    fn new(duration: Duration) -> Self {
        Self {
            duration,
            started_at: None,
            waker_sent: Arc::new(Mutex::new(false)),
        }
    }
}

impl Future for NonBlockingSleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if already completed
        if *self.waker_sent.lock().unwrap() {
            return Poll::Ready(());
        }

        // Start the timer on first poll
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
            let duration = self.duration;
            let waker = cx.waker().clone();
            let waker_sent = Arc::clone(&self.waker_sent);

            thread::spawn(move || {
                thread::sleep(duration);
                *waker_sent.lock().unwrap() = true;
                waker.wake();
            });
        }

        // Still not ready, but waker is already registered
        Poll::Pending
    }
}

fn main() {
    println!("=== Blocking vs Non-Blocking Comparison ===\n");

    // ===== ブロッキング版（悪い例）=====
    println!("--- BAD: Blocking sleep (blocks worker thread) ---");
    let start = Instant::now();
    {
        let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

        println!("Submitting 3 tasks that sleep 100ms each (BLOCKING)");
        let r1 = engine.reserve(async {
            println!("  Task 1 starting... (will BLOCK worker)");
            std::thread::sleep(Duration::from_millis(100));
            println!("  Task 1 done!");
            1
        });

        let r2 = engine.reserve(async {
            println!("  Task 2 starting... (will BLOCK worker)");
            std::thread::sleep(Duration::from_millis(100));
            println!("  Task 2 done!");
            2
        });

        let r3 = engine.reserve(async {
            println!("  Task 3 starting... (will BLOCK worker)");
            std::thread::sleep(Duration::from_millis(100));
            println!("  Task 3 done!");
            3
        });

        block_on(r1);
        block_on(r2);
        block_on(r3);

        engine.graceful_shutdown();
    }
    let blocking_time = start.elapsed();
    println!("Blocking version took: {:?}", blocking_time);
    println!("(Notice: tasks run sequentially even with 2 workers!)\n");

    // ===== 非ブロッキング版（良い例）=====
    println!("--- GOOD: Non-blocking sleep (uses Waker) ---");
    let start = Instant::now();
    {
        let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

        println!("Submitting 3 tasks that sleep 100ms each (NON-BLOCKING)");
        let r1 = engine.reserve(async {
            println!("  Task 1 starting... (non-blocking)");
            NonBlockingSleep::new(Duration::from_millis(100)).await;
            println!("  Task 1 done!");
            1
        });

        let r2 = engine.reserve(async {
            println!("  Task 2 starting... (non-blocking)");
            NonBlockingSleep::new(Duration::from_millis(100)).await;
            println!("  Task 2 done!");
            2
        });

        let r3 = engine.reserve(async {
            println!("  Task 3 starting... (non-blocking)");
            NonBlockingSleep::new(Duration::from_millis(100)).await;
            println!("  Task 3 done!");
            3
        });

        block_on(r1);
        block_on(r2);
        block_on(r3);

        engine.graceful_shutdown();
    }
    let nonblocking_time = start.elapsed();
    println!("Non-blocking version took: {:?}", nonblocking_time);
    println!("(Notice: tasks run concurrently!)\n");

    println!("=== Summary ===");
    println!("Blocking:     {:?} (tasks waited in queue)", blocking_time);
    println!("Non-blocking: {:?} (tasks ran concurrently)", nonblocking_time);
    println!("\nSpeed improvement: {:.1}x faster!",
             blocking_time.as_secs_f64() / nonblocking_time.as_secs_f64());
}
