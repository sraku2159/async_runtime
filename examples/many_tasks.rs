use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

fn main() {
    println!("=== Many Tasks Example ===\n");

    let mut engine = Engine::new(8, |receiver| Box::new(Fifo::new(receiver)));

    println!("Created engine with 8 workers");
    println!("Spawning 100 tasks...\n");

    let mut receivers = vec![];

    for i in 0..100 {
        let receiver = engine.reserve(async move {
            let result = i * i;

            let a = async { 42 };
            let b = async { 43 };
            let result = result + a.await + b.await;
            if i % 10 == 0 {
                println!("  Task {} completed: {} * {} = {}", i, i, i, result);
            }
            result
        }, None);
        receivers.push((i, receiver));
    }

    println!("\nWaiting for all tasks to complete...\n");

    for (i, receiver) in receivers {
        let result = block_on(receiver);
        assert_eq!(result, i * i + 85, "Task {} produced wrong result", i);
    }

    println!("All 100 tasks completed successfully!");

    println!("\nShutting down engine...");
    engine.graceful_shutdown();
    println!("Done!");
}
