use std::future::Future;

pub trait Scheduler {
    type Output;
    fn schedule() -> dyn Future<Output = Self::Output>;
}
