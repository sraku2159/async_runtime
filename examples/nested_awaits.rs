use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

fn main() {
    println!("=== Nested Awaits Example ===\n");
    println!("Testing deeply nested async/await chains\n");

    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    // Level 1: Simple async task
    let level1 = engine.reserve(async {
        println!("[Level 1] Starting...");
        42
    }, None);

    // Level 2: Await level 1
    let level2 = engine.reserve(async move {
        println!("[Level 2] Starting...");
        let result1 = block_on(level1);
        println!("[Level 2] Got from level 1: {}", result1);
        result1 + 10
    }, None);

    // Level 3: Await level 2
    let level3 = engine.reserve(async move {
        println!("[Level 3] Starting...");
        let result2 = block_on(level2);
        println!("[Level 3] Got from level 2: {}", result2);
        result2 * 2
    }, None);

    // Level 4: Await level 3
    let level4 = engine.reserve(async move {
        println!("[Level 4] Starting...");
        let result3 = block_on(level3);
        println!("[Level 4] Got from level 3: {}", result3);
        result3 + 100
    }, None);

    // Level 5: Multiple awaits in sequence
    let level5 = engine.reserve(async move {
        println!("[Level 5] Starting...");

        let step1 = async { 5 };
        let step2 = async { 10 };
        let step3 = async { 15 };

        let a = step1.await;
        println!("[Level 5] Step 1: {}", a);

        let b = step2.await;
        println!("[Level 5] Step 2: {}", b);

        let c = step3.await;
        println!("[Level 5] Step 3: {}", c);

        let result4 = block_on(level4);
        println!("[Level 5] Got from level 4: {}", result4);

        result4 + a + b + c
    }, None);

    // Parallel branch 1
    let branch1 = engine.reserve(async {
        println!("[Branch 1] Starting...");

        let sub1 = async { 1 };
        let sub2 = async { 2 };
        let sub3 = async { 3 };

        let x = sub1.await;
        let y = sub2.await;
        let z = sub3.await;

        println!("[Branch 1] Computed: {} + {} + {} = {}", x, y, z, x + y + z);
        x + y + z
    }, None);

    // Parallel branch 2
    let branch2 = engine.reserve(async {
        println!("[Branch 2] Starting...");

        let sub1 = async { 10 };
        let sub2 = async { 20 };
        let sub3 = async { 30 };

        let x = sub1.await;
        let y = sub2.await;
        let z = sub3.await;

        println!("[Branch 2] Computed: {} + {} + {} = {}", x, y, z, x + y + z);
        x + y + z
    }, None);

    // Final level: Await everything
    let final_result = engine.reserve(async move {
        println!("[Final] Starting...");

        let main_result = block_on(level5);
        println!("[Final] Main chain result: {}", main_result);

        let b1 = block_on(branch1);
        println!("[Final] Branch 1 result: {}", b1);

        let b2 = block_on(branch2);
        println!("[Final] Branch 2 result: {}", b2);

        // One more nested layer
        let bonus = async {
            let inner1 = async { 7 };
            let inner2 = async { 8 };
            inner1.await + inner2.await
        };

        let bonus_result = bonus.await;
        println!("[Final] Bonus computation: {}", bonus_result);

        main_result + b1 + b2 + bonus_result
    }, None);

    println!("\nWaiting for final result...\n");
    let result = block_on(final_result);

    println!("\n=== Results ===");
    println!("Final result: {}", result);
    println!("Expected: {} (42 + 10) * 2 + 100 + 5 + 10 + 15 + 6 + 60 + 15 = {}",
             "(42 + 10) * 2 + 100 + 5 + 10 + 15",
             (42 + 10) * 2 + 100 + 5 + 10 + 15 + 6 + 60 + 15);

    assert_eq!(result, 315, "Result mismatch!");
    println!("✓ All nested awaits executed correctly!");

    println!("\nShutting down engine...");
    engine.graceful_shutdown();
    println!("Done!");
}
