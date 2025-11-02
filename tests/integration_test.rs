use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;

#[test]
fn engine_executes_simple_task() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async { 1 + 1 });

    let result = block_on(receiver);
    assert_eq!(result, 2);

    engine.graceful_shutdown();
}

#[test]
fn engine_executes_multiple_tasks() {
    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    let r1 = engine.reserve(async { 10 });
    let r2 = engine.reserve(async { 20 });
    let r3 = engine.reserve(async { 30 });

    assert_eq!(block_on(r1), 10);
    assert_eq!(block_on(r2), 20);
    assert_eq!(block_on(r3), 30);

    engine.graceful_shutdown();
}

#[test]
fn engine_executes_async_computation() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async {
        let a = 5;
        let b = 7;
        a * b
    });

    let result = block_on(receiver);
    assert_eq!(result, 35);

    engine.graceful_shutdown();
}

#[test]
fn engine_handles_string_results() {
    let mut engine = Engine::new(2, |receiver| Box::new(Fifo::new(receiver)));

    let receiver = engine.reserve(async { "Hello, async runtime!".to_string() });

    let result = block_on(receiver);
    assert_eq!(result, "Hello, async runtime!");

    engine.graceful_shutdown();
}

#[test]
fn engine_with_single_worker() {
    let mut engine = Engine::new(1, |receiver| Box::new(Fifo::new(receiver)));

    let r1 = engine.reserve(async { 100 });
    let r2 = engine.reserve(async { 200 });

    assert_eq!(block_on(r1), 100);
    assert_eq!(block_on(r2), 200);

    engine.graceful_shutdown();
}

#[test]
fn engine_with_many_workers() {
    let mut engine = Engine::new(8, |receiver| Box::new(Fifo::new(receiver)));

    let mut receivers = vec![];
    for i in 0..10 {
        let receiver = engine.reserve(async move { i * 2 });
        receivers.push(receiver);
    }

    for (i, receiver) in receivers.into_iter().enumerate() {
        assert_eq!(block_on(receiver), i * 2);
    }

    engine.graceful_shutdown();
}
