use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::{pin::Pin, sync::atomic::AtomicU8, task::Poll};

pub const IDLE: u8 = 0;
pub const SCHEDULED: u8 = 1;
pub const RUNNING: u8 = 2;
pub const COMPLETED: u8 = 3;

pub trait TaskTrait: Future<Output = ()> {
    fn set_state(&mut self, val: u8);
    fn get_state(&self) -> u8;
    fn is_scheduled(&self) -> bool {
        self.get_state() == SCHEDULED
    }
}

pub struct Task<T> {
    inner: Pin<Box<dyn Future<Output = ()>>>,
    state: AtomicU8,
    // sender: Sender<T>
    // TODO: これはsenderの実装がおわるまでエラーを消すため
    phantom: PhantomData<T>,
}

impl<T> Task<T>
where
    T: Unpin + 'static,
{
    pub fn new<U>(inner: U) -> Self
    where
        U: Future + 'static,
    {
        let inner = Box::pin(inner);
        let task = async move {
            let _res = inner.await;
            // sender.send(res);
        };
        Self {
            inner: Box::pin(task),
            state: AtomicU8::new(IDLE),
            phantom: PhantomData,
        }
    }
}

impl<T> TaskTrait for Task<T>
where
    T: Unpin + 'static,
{
    fn set_state(&mut self, val: u8) {
        self.state.store(val, Ordering::Release);
    }

    fn get_state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    fn is_scheduled(&self) -> bool {
        self.state.load(Ordering::Acquire) == SCHEDULED
    }
}

impl<T> Future for Task<T>
where
    T: Unpin + 'static,
{
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

// impl<T> Future for Task<T> {
//     type Output = ();
//
//     fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
//         //
//         let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
//         let state = unsafe { &self.get_unchecked_mut().state };
//
//         match inner.poll(cx) {
//             Poll::Pending => Poll::Pending,
//             Poll::Ready(v) => {
//                 // self.sender(v);
//                 state.store(COMPLETED, Ordering::Release);
//                 Poll::Ready(v)
//             }
//         }
//     }
// }
