pub mod schedule;
pub mod waker;
pub mod worker;

use schedule::Scheduler;
use std::{
    future::Future,
    pin::{Pin, pin},
};
// use waker::*;
use worker::Worker;

pub struct Engine<T, U>
where
    T: Scheduler,
    U: Worker,
{
    scheduler: T,
    worker: U,
}

impl<T, U> Engine<T, U>
where
    T: Scheduler,
    U: Worker,
{
    // workerのインスタンスかがされた時点で、別スレッドで起動してる
    pub fn new(scheduler: T, worker: U) -> Self {
        Self { scheduler, worker }
    }

    pub fn execute(&self) {}

    pub fn reserve<V>(&self, _task: Pin<Box<dyn Future<Output = V>>>) {
        todo!();
        // let task = async move {
        //     let res = task.await;
        //     sender.send(res);
        // };
        //
        // let task = pin!(Box::new(task));
        // self.scheduler.add(task);
    }
}
