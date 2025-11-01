use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::{pin::Pin, sync::atomic::AtomicU8, task::Poll};

use crate::utils::channel::Sender;

pub const IDLE: u8 = 0;
pub const SCHEDULED: u8 = 1;
pub const RUNNING: u8 = 2;
pub const COMPLETED: u8 = 3;

pub trait TaskTrait: Future<Output = ()> {
    fn set_state(self: Pin<&mut Self>, val: u8);
    fn get_state(self: Pin<&Self>) -> u8;
    fn is_scheduled(self: Pin<&Self>) -> bool {
        self.get_state() == SCHEDULED
    }
}

pub struct Task {
    inner: Pin<Box<dyn Future<Output = ()>>>,
    state: AtomicU8,
    // TODO: これはsenderの実装がおわるまでエラーを消すため
}

impl Task {
    pub fn new<T, U>(inner: T, sender: Sender<U>) -> Self
    where
        T: Future<Output = U> + 'static,
        U: Clone + Unpin + 'static,
    {
        let inner = Box::pin(inner);
        let task = async move {
            let res = inner.await;
            sender.send(res);
            //ここでreceiverのFutureをwakeする必要がある。
        };
        Self {
            inner: Box::pin(task),
            state: AtomicU8::new(IDLE),
        }
    }
}

impl TaskTrait for Task {
    fn set_state(self: Pin<&mut Self>, val: u8) {
        self.get_mut().state.store(val, Ordering::Release);
    }

    fn get_state(self: Pin<&Self>) -> u8 {
        self.state.load(Ordering::Acquire)
    }
}

impl Future for Task {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let pinned = self.get_mut();
        match pinned.inner.as_mut().poll(cx) {
            Poll::Pending => {
                pinned.state.store(IDLE, Ordering::Release);
                Poll::Pending
            }
            Poll::Ready(v) => {
                // self.sender(v);
                pinned.state.store(COMPLETED, Ordering::Release);
                Poll::Ready(v)
            }
        }
    }
}
