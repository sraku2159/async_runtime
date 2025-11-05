use async_runtime::{
    Engine,
    engine::{schedule::deadline::DeadLineScheduler, block_on},
};

fn main() {
    println!("=== Deadline Scheduler Example ===\n");
    println!("This example demonstrates the deadline scheduler.");
    println!("Tasks are executed in order of their deadlines (smallest first).\n");

    let mut engine = Engine::new(2, |receiver| Box::new(DeadLineScheduler::new(receiver)));

    println!("Submitting tasks with different deadlines:");

    let task1 = engine.reserve(async {
        println!("  Task 1: deadline=100");
        100
    }, Some(100));

    let task2 = engine.reserve(async {
        println!("  Task 2: deadline=300");
        300
    }, Some(300));

    let task3 = engine.reserve(async {
        println!("  Task 3: deadline=200");
        200
    }, Some(200));

    let task4 = engine.reserve(async {
        println!("  Task 4: no deadline");
        0
    }, None);

    println!("\nWaiting for results...\n");

    let r1 = block_on(task1);
    let r2 = block_on(task2);
    let r3 = block_on(task3);
    let r4 = block_on(task4);

    println!("\n=== Results ===");
    println!("Task 1 (deadline=100): {}", r1);
    println!("Task 2 (deadline=300): {}", r2);
    println!("Task 3 (deadline=200): {}", r3);
    println!("Task 4 (no deadline): {}", r4);
    println!("\nNote: Execution order depends on scheduling and may vary.");
    println!("For deterministic ordering tests, see tests/integration_test.rs");

    engine.graceful_shutdown();
}
