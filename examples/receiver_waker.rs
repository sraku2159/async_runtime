use std::thread;
use std::time::Duration;

use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

fn main() {
    println!("=== Receiver Waker Example ===\n");
    println!("This demonstrates that sender.send() calls the receiver's waker\n");

    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    println!("Submitting task that sleeps before returning...");
    let receiver = engine.reserve(async {
        println!("  Task started in worker thread");
        // 少し待ってから値を返す（receiverがWakerを登録する時間を確保）
        thread::sleep(Duration::from_secs(1));
        println!("  Task returning value 42");
        42
    });

    println!("Main thread: calling block_on(receiver)...");
    println!("Main thread: receiver will return Pending first, then sender will wake it up\n");

    let result = block_on(receiver);

    println!("\nMain thread: Got result: {}", result);
    println!("\nLook for '[Sender] Calling waker to wake up receiver!' in the output above!");

    engine.graceful_shutdown();
}
