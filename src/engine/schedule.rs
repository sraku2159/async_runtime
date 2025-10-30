pub mod fifo;

use std::future::Future;
use std::pin::Pin;

pub trait Scheduler {
    fn schedule(&mut self, task: Pin<Box<dyn Future<Output = ()>>>);
    fn take(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>>;
}
