pub use std::future::Future;
pub use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Waker;
pub use std::task::{Context, Poll};

pub use std::sync::mpsc;

pub type Channel<T> = (Sender<T>, Receiver<T>);
pub fn channel<T>() -> Channel<T>
where
    T: Clone,
{
    let (sender, receiver) = mpsc::channel();
    let shared_context = Arc::new(Mutex::new(InnerContext::new()));
    (
        Sender::new(sender, shared_context.clone()),
        Receiver::new(receiver, shared_context.clone()),
    )
}

pub struct Sender<T>
where
    T: Clone,
{
    sender: mpsc::Sender<T>,
    context: SharedInnerContext,
}

impl<T> Sender<T>
where
    T: Clone,
{
    fn new(sender: mpsc::Sender<T>, context: SharedInnerContext) -> Self {
        Self {
            sender: sender,
            context: context,
        }
    }

    pub fn send(self, val: T) {
        let _ = self.sender.send(val.clone());
        let mut context = self.context.lock().unwrap();
        context.set_state(InnerState::Ready);
        if let Some(waker) = context.waker.take() {
            eprintln!("[Sender] Calling waker to wake up receiver!");
            waker.wake();
        }
    }
}

pub struct Receiver<T>
where
    T: Clone,
{
    receiver: mpsc::Receiver<T>,
    context: SharedInnerContext,
}

impl<T> Receiver<T>
where
    T: Clone,
{
    fn new(receiver: mpsc::Receiver<T>, shared_context: SharedInnerContext) -> Self {
        Self {
            receiver: receiver,
            context: shared_context,
        }
    }

    pub fn set_state(&mut self, state: InnerState) -> () {
        self.context.lock().unwrap().set_state(state)
    }
}

impl<T> Future for Receiver<T>
where
    T: Clone,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut context = self.context.lock().unwrap();
        match context.state {
            InnerState::Pending => {
                match &mut context.waker {
                    Some(waker) => waker.clone_from(cx.waker()),
                    None => context.waker = Some(cx.waker().clone()),
                }
                Poll::Pending
            }
            InnerState::Ready => Poll::Ready(self.receiver.recv().unwrap()),
        }
    }
}

type SharedInnerContext = Arc<Mutex<InnerContext>>;

struct InnerContext {
    state: InnerState,
    waker: Option<Waker>,
}

impl InnerContext {
    // NOTE: もしかしたらstate欲しいとなる可能性もあるので、注意
    fn new() -> Self {
        Self {
            // NOTE: 困るかも？
            state: InnerState::Pending,
            waker: None,
        }
    }

    fn set_state(&mut self, state: InnerState) {
        self.state = state;
    }
}

pub enum InnerState {
    Pending,
    Ready,
}

#[cfg(test)]
mod test;
