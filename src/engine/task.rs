use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::{pin::Pin, sync::atomic::AtomicU8, task::Poll};

use crate::utils::channel::Sender;

pub const PENDING: u8 = 0;
pub const SCHEDULED: u8 = 1;
pub const RUNNING: u8 = 2;
pub const COMPLETED: u8 = 3;

pub type SharedTask = Arc<Task>;

fn state_name(state: u8) -> &'static str {
    match state {
        PENDING => "PENDING",
        SCHEDULED => "SCHEDULED",
        RUNNING => "RUNNING",
        COMPLETED => "COMPLETED",
        _ => "UNKNOWN",
    }
}

pub struct Task {
    inner: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    state: AtomicU8,
    deadline: Option<u64>,
}

impl Task {
    pub fn new<T, U>(inner: T, sender: Sender<U>, deadline: Option<u64>) -> SharedTask
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
            deadline,
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
        // SCHEDULED -> RUNNING の遷移のみ許可
        // これによりCOMPLETEDやRUNNING中のタスクがpollされるのを防ぐ
        let before_state = self.state.load(Ordering::Acquire);
        eprintln!("[Task::poll] Before: state={}", state_name(before_state));

        match self
            .state
            .compare_exchange(SCHEDULED, RUNNING, Ordering::Release, Ordering::Acquire)
        {
            Ok(_) => {
                eprintln!("[Task::poll] State transition: SCHEDULED -> RUNNING");
                // 状態遷移成功、pollを実行
                let mut inner = self.inner.lock().unwrap();
                match inner.as_mut().poll(cx) {
                    Poll::Pending => {
                        eprintln!("[Task::poll] Future returned Pending");
                        // RUNNING -> PENDING の遷移を試みる
                        // もしwake()が既に呼ばれてSCHEDULEDになっていたら、そのまま
                        match self.state.compare_exchange(
                            RUNNING,
                            PENDING,
                            Ordering::Release,
                            Ordering::Acquire,
                        ) {
                            Ok(_) => eprintln!("[Task::poll] State transition: RUNNING -> PENDING"),
                            Err(actual) => eprintln!(
                                "[Task::poll] State transition failed: RUNNING -> PENDING (actual: {})",
                                state_name(actual)
                            ),
                        }
                        Poll::Pending
                    }
                    Poll::Ready(v) => {
                        eprintln!("[Task::poll] Future returned Ready");
                        self.state.store(COMPLETED, Ordering::Release);
                        eprintln!("[Task::poll] State transition: RUNNING -> COMPLETED");
                        Poll::Ready(v)
                    }
                }
            }
            Err(actual) => {
                eprintln!(
                    "[Task::poll] State transition failed: SCHEDULED -> RUNNING (actual: {})",
                    state_name(actual)
                );
                // 状態遷移失敗（既にRUNNINGかCOMPLETED）
                // Pendingを返して何もしない
                Poll::Pending
            }
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deadline.partial_cmp(&other.deadline)
    }
}
