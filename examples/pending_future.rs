use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

/// Poll::Pendingを返すカスタムFuture
/// cnt が max_polls に達するまで Poll::Pending を返し続ける
struct CountingFuture {
    cnt: usize,
    max_polls: usize,
    value: i32,
}

impl CountingFuture {
    fn new(max_polls: usize, value: i32) -> Self {
        println!("Creating CountingFuture: will poll {} times before returning {}", max_polls, value);
        Self {
            cnt: 0,
            max_polls,
            value,
        }
    }
}

impl Future for CountingFuture {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.cnt += 1;
        println!("  CountingFuture poll attempt #{} (max: {})", self.cnt, self.max_polls);

        if self.cnt >= self.max_polls {
            println!("  -> Ready! Returning {}", self.value);
            Poll::Ready(self.value)
        } else {
            println!("  -> Pending... calling waker to reschedule");
            // Poll::Pendingを返す前にWakerを呼んで再スケジュールを要求
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn main() {
    println!("=== Pending Future Example ===\n");

    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    println!("Submitting task 1: CountingFuture(3 polls, value=100)");
    let r1 = engine.reserve(CountingFuture::new(3, 100));

    println!("\nSubmitting task 2: CountingFuture(5 polls, value=200)");
    let r2 = engine.reserve(CountingFuture::new(5, 200));

    println!("\nSubmitting task 3: CountingFuture(2 polls, value=300)");
    let r3 = engine.reserve(CountingFuture::new(2, 300));

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
