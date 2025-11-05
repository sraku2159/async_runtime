use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

fn main() {
    println!("=== Nested Task Waker Example ===\n");
    println!("This demonstrates that a task's Waker is called when it awaits another task\n");

    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    println!("Creating inner task...");
    let inner_task = engine.reserve(async {
        println!("  [Inner task] Running in worker thread");
        // 少し待ってから値を返す（outer taskがWakerを登録する時間を確保）
        std::thread::sleep(std::time::Duration::from_millis(10));
        println!("  [Inner task] Returning result");
        42
    }, None);

    println!("Creating outer task that awaits inner task...");
    let outer_task = engine.reserve(async move {
        println!("  [Outer task] Starting, will await inner task");
        println!("  [Outer task] Calling inner_task.await (will return Pending first)...");

        let result = inner_task.await;

        println!("  [Outer task] Inner task completed with result: {}", result);
        println!("  [Outer task] Waker was called to wake us up!");
        result * 2
    }, None);

    println!("\nMain thread: Waiting for outer task...");
    let final_result = block_on(outer_task);

    println!("\nFinal result: {}", final_result);
    println!("\nLook for 'called waker' and 'task reshceduled!!' in the output!");
    println!("These show that the outer task's Waker was called when inner task completed.");

    engine.graceful_shutdown();
}
