use crate::engine::waker::Waker;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub trait Worker {
    fn execute<T>(&self, task: Pin<Box<dyn Future<Output = T>>>) -> Poll<T>
    where
        T: Future;
}
//
// pub struct MultiThreadWorker {
//     pool: WorkerPool,
// }
//
// impl Worker for MultiThreadWorker {
//     fn execute<T>(&self, task: &mut Pin<Box<dyn Future<Output = T>>>) -> Poll<T> {
//         let waker = Arc::new(Waker::new()).into();
//         let mut cx = Context::from_waker(&waker);
//         task.as_mut().poll(&mut cx)
//     }
// }
//
// pub struct WorkerPool {}
