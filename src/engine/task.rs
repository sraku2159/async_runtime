use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::{pin::Pin, sync::atomic::AtomicU8, task::Poll};

use crate::utils::channel::Sender;

pub const PENDING: u8 = 0;
pub const SCHEDULED: u8 = 1;
pub const RUNNING: u8 = 2;
pub const COMPLETED: u8 = 3;

pub type SharedTask = Arc<Task>;

pub struct Task {
    inner: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    state: AtomicU8,
}

impl Task {
    pub fn new<T, U>(inner: T, sender: Sender<U>) -> SharedTask
    where
        T: Future<Output = U> + Send + 'static,
        U: Clone + Unpin + Send + 'static,
    {
        let inner = Box::pin(inner);
        let task = async move {
            let res = inner.await;
            sender.send(res);
        };
        Arc::new(Self {
            inner: Mutex::new(Box::pin(task)),
            state: AtomicU8::new(PENDING),
        })
    }

    pub fn set_state(&self, val: u8) {
        self.state.store(val, Ordering::Release);
    }

    pub fn get_state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    pub fn is_scheduled(&self) -> bool {
        self.get_state() == SCHEDULED
    }

    pub fn poll(&self, cx: &mut std::task::Context<'_>) -> Poll<()> {
        let mut inner = self.inner.lock().unwrap();
        match inner.as_mut().poll(cx) {
            Poll::Pending => {
                self.state.store(PENDING, Ordering::Release);
                Poll::Pending
            }
            Poll::Ready(v) => {
                self.state.store(COMPLETED, Ordering::Release);
                Poll::Ready(v)
            }
        }
    }
}
