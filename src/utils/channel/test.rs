use crate::utils::channel::{InnerContext, Sender};

use super::channel;
use std::{
    pin::pin,
    sync::mpsc,
    task::{Context, Poll, Wake, Waker},
};

#[test]
fn sender_call_waker() {
    use std::sync::{Arc, Mutex};

    struct MockWaker(Mutex<i64>);

    impl Wake for MockWaker {
        fn wake(self: std::sync::Arc<Self>) {
            let mut called_cnt = self.0.lock().unwrap();
            *(called_cnt) += 1;
            assert_eq!(*called_cnt, 1);
        }
    }

    let waker = Waker::from(Arc::new(MockWaker(Mutex::new(0))));
    let mut inner = InnerContext::new();
    inner.waker = Some(waker);
    let (sender, _) = mpsc::channel();
    let sender = Sender::new(sender, Arc::new(Mutex::new(inner)));

    sender.send(());
}

#[test]
fn receiver_return_pending() {
    let (_, receiver) = channel::<()>();

    let mut receiver = pin!(receiver);
    let mut context = Context::from_waker(Waker::noop());

    let receiver = receiver.as_mut();
    let res = receiver.poll(&mut context);
    assert_eq!(res, Poll::Pending);
}

#[test]
fn receiver_return_ready() {
    let (sender, receiver) = channel::<i64>();

    let mut receiver = pin!(receiver);
    let mut context = Context::from_waker(Waker::noop());

    sender.send(42);

    let receiver = receiver.as_mut();
    let res = receiver.poll(&mut context);
    assert_eq!(res, Poll::Ready(42));
}
