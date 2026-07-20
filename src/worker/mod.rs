use std::time::Duration;

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::model::Task;

#[derive(Debug, Clone)]
pub struct WorkerState {
    pub current_load: u32,
    pub is_free: bool,
    pub is_busy: bool,
    pub processed_tasks: u64,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            current_load: 0,
            is_free: true,
            is_busy: false,
            processed_tasks: 0,
        }
    }
}

pub struct Worker {
    worker_id: String,
    state: std::sync::Arc<std::sync::Mutex<WorkerState>>,
    receiver: mpsc::Receiver<Task>,
}

impl Worker {
    pub fn new(worker_id: impl Into<String>, receiver: mpsc::Receiver<Task>) -> Self {
        Self {
            worker_id: worker_id.into(),
            state: std::sync::Arc::new(std::sync::Mutex::new(WorkerState::default())),
            receiver,
        }
    }

    pub fn state_handle(&self) -> std::sync::Arc<std::sync::Mutex<WorkerState>> {
        self.state.clone()
    }

    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run().await;
        })
    }

    async fn run(mut self) {
        while let Some(task) = self.receiver.recv().await {
            {
                let mut state = self.state.lock().expect("worker state lock poisoned");
                state.current_load = 1;
                state.is_free = false;
                state.is_busy = true;
            }

            let delay = Self::compute_processing_delay(&task);
            sleep(delay).await;

            {
                let mut state = self.state.lock().expect("worker state lock poisoned");
                state.current_load = 0;
                state.is_free = true;
                state.is_busy = false;
                state.processed_tasks += 1;
            }
        }
    }

    fn compute_processing_delay(task: &Task) -> Duration {
        let payload_factor = (task.payload.bytes.len() as u64 / 256) * 8;
        let sequence_factor = (task.metadata.request_sequence % 5) * 20;
        let tenant_factor = (task.tenant_id.len() as u64 % 4) * 15;
        let delay_ms = 40 + payload_factor + sequence_factor + tenant_factor;
        Duration::from_millis(delay_ms.max(40))
    }
}

pub struct WorkerHandle {
    pub worker_id: String,
    pub state: std::sync::Arc<std::sync::Mutex<WorkerState>>,
    sender: mpsc::Sender<Task>,
    pub join_handle: JoinHandle<()>,
}

impl WorkerHandle {
    pub fn submit(&self, task: Task) -> Result<()> {
        self.sender
            .try_send(task)
            .map_err(|error| anyhow!("worker {} submit failed: {}", self.worker_id, error))
    }

    pub fn snapshot_state(&self) -> WorkerState {
        self.state
            .lock()
            .expect("worker state lock poisoned")
            .clone()
    }

    pub fn close(self) {
        drop(self.sender);
    }
}

pub fn spawn_worker(worker_id: impl Into<String>, queue_capacity: usize) -> WorkerHandle {
    let (sender, receiver) = mpsc::channel(queue_capacity);
    let worker = Worker::new(worker_id.into(), receiver);
    let state = worker.state_handle();
    let worker_id = worker.worker_id.clone();
    let join_handle = worker.spawn();

    WorkerHandle {
        worker_id,
        state,
        sender,
        join_handle,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::{sleep, timeout};

    use crate::model::{NetworkPayload, Task, TrackingMetadata};

    use super::spawn_worker;

    #[tokio::test]
    async fn worker_processes_tasks_and_updates_state_flags() {
        let worker = spawn_worker("worker_a", 8);

        let task_one = Task::new(
            "tenant_a",
            NetworkPayload::new(vec![1; 256], "application/octet-stream"),
            TrackingMetadata::new(1_000, 1),
        );
        let task_two = Task::new(
            "tenant_a",
            NetworkPayload::new(vec![2; 1024], "application/octet-stream"),
            TrackingMetadata::new(1_100, 2),
        );

        worker
            .submit(task_one)
            .expect("first task should be queued");
        worker
            .submit(task_two)
            .expect("second task should be queued");

        timeout(Duration::from_secs(3), async {
            loop {
                let snapshot = worker.snapshot_state();
                if snapshot.processed_tasks >= 2 {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("worker did not process tasks in time");

        let final_state = worker.snapshot_state();
        assert!(final_state.is_free);
        assert!(!final_state.is_busy);
        assert_eq!(final_state.current_load, 0);
        assert_eq!(final_state.processed_tasks, 2);

        worker.close();
    }
}
