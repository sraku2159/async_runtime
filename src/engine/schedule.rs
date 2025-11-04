pub mod deadline;
pub mod fifo;
use crate::engine::task::SharedTask;
use crate::engine::worker::WorkerInfo;
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

pub trait Scheduler {
    // スケジューリングアルゴリズムの実装：タスクを登録
    fn register(&mut self, task: SharedTask);

    // スケジューリングアルゴリズムの実装：次に実行するタスクを取得
    fn take(&mut self) -> Option<SharedTask>;

    // pending_workersへのアクセス
    fn get_pending_workers(&mut self) -> &mut VecDeque<WorkerInfo>;

    // worker_receiverへのアクセス
    fn get_worker_receiver(&mut self) -> &mut Receiver<WorkerInfo>;

    // デフォルト実装：タスクをスケジュールし、通知
    fn schedule(&mut self, task: SharedTask) {
        eprintln!("[Scheduler::schedule] Setting task state to SCHEDULED");
        task.set_state(crate::engine::task::SCHEDULED);
        self.register(task);
        self.notify();
    }

    // デフォルト実装：新しいWorkerInfoを収集し、タスクを配布
    fn notify(&mut self) {
        // 新しい WorkerInfo を全て取得
        while let Ok(worker_info) = self.get_worker_receiver().try_recv() {
            self.get_pending_workers().push_back(worker_info);
        }

        // pending_workers からタスクを配布
        while let Some(worker_info) = self.get_pending_workers().pop_front() {
            if let Some(task) = self.take() {
                let _ = worker_info.sender.send(task);
                worker_info.t.unpark();
            } else {
                // タスクがないので WorkerInfo を戻す
                self.get_pending_workers().push_front(worker_info);
                break;
            }
        }
    }
}

impl Scheduler for Box<dyn Scheduler + Send> {
    fn register(&mut self, task: SharedTask) {
        (**self).register(task)
    }

    fn take(&mut self) -> Option<SharedTask> {
        (**self).take()
    }

    fn get_pending_workers(&mut self) -> &mut VecDeque<WorkerInfo> {
        (**self).get_pending_workers()
    }

    fn get_worker_receiver(&mut self) -> &mut Receiver<WorkerInfo> {
        (**self).get_worker_receiver()
    }
}
