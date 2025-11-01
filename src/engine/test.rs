use super::block_on;

use std::future::Future;
use std::task::Poll;

#[test]
fn block_on_normally() {
    struct DummyFuture {
        i: i32,
    }
    impl Future for DummyFuture {
        type Output = i32;
        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> Poll<Self::Output> {
            if self.i < 3 {
                self.i += 1;
                let waker = cx.waker().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    waker.wake();
                });
                Poll::Pending
            } else {
                Poll::Ready(42)
            }
        }
    }

    let f = Box::pin(DummyFuture { i: 0 });

    let res = block_on(f);

    assert_eq!(res, 42);
}
