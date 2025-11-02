use async_runtime::engine::schedule::fifo::Fifo;
use async_runtime::engine::block_on;
use async_runtime::Engine;

fn main() {
    println!("=== Different Return Types Example ===\n");

    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    println!("Testing different return types:\n");

    // Integer
    println!("1. Integer task");
    let r_int = engine.reserve(async { 42 });

    // String
    println!("2. String task");
    let r_string = engine.reserve(async { "Async is awesome!".to_string() });

    // Boolean
    println!("3. Boolean task");
    let r_bool = engine.reserve(async { true });

    // Float
    println!("4. Float task");
    let r_float = engine.reserve(async { 3.14159 });

    // Tuple
    println!("5. Tuple task");
    let r_tuple = engine.reserve(async { (1, "two".to_string(), 3.0) });

    // Vec
    println!("6. Vector task");
    let r_vec = engine.reserve(async { vec![1, 2, 3, 4, 5] });

    println!("\nResults:");
    println!("  Integer: {}", block_on(r_int));
    println!("  String: {}", block_on(r_string));
    println!("  Boolean: {}", block_on(r_bool));
    println!("  Float: {}", block_on(r_float));
    println!("  Tuple: {:?}", block_on(r_tuple));
    println!("  Vector: {:?}", block_on(r_vec));

    println!("\nShutting down engine...");
    engine.graceful_shutdown();
    println!("Done!");
}
