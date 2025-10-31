pub mod fifo;

use std::future::Future;
use std::pin::Pin;

pub trait Scheduler {
    fn schedule<T>(&mut self, task: Pin<Box<dyn Future<Output = T>>>)
    where
        T: Unpin + 'static;
    fn take(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>>;
}
