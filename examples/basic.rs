use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

fn main() {
    println!("=== Basic Async Runtime Example ===\n");

    // Create engine with FIFO scheduler and 4 workers
    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    println!("Created engine with 4 workers\n");

    // Simple computation
    println!("Running simple computation...");
    let r1 = engine.reserve(async {
        println!("  [Task 1] Computing 5 + 3");
        5 + 3
    });

    // String processing
    println!("Running string task...");
    let r2 = engine.reserve(async {
        println!("  [Task 2] Creating greeting");
        "Hello from async runtime!".to_string()
    });

    // More complex computation
    println!("Running complex computation...");
    let r3 = engine.reserve(async {
        println!("  [Task 3] Computing factorial of 5");
        let mut result = 1;
        for i in 1..=5 {
            result *= i;
        }
        result
    });

    // Collect results
    println!("\nCollecting results:");
    println!("  Task 1 result: {}", block_on(r1));
    println!("  Task 2 result: {}", block_on(r2));
    println!("  Task 3 result: {}", block_on(r3));

    println!("\nShutting down engine...");
    engine.graceful_shutdown();
    println!("Done!");
}
