use futures::executor::ThreadPool;
use futures::Future;
use futures::task::{FutureObj, SpawnExt};
use futures::prelude::*;
use future::RemoteHandle;

trait AssertSendSync: Send + Sync {}

#[derive(Clone)]
pub struct TaskSpawner {
    pub pool_handle: ThreadPool,
}

impl AssertSendSync for TaskSpawner {}

impl From<ThreadPool> for TaskSpawner {
    /// Consume a ThreadPool to create a TaskSpawner.
    fn from(pool: ThreadPool) -> Self {
        TaskSpawner {
            pool_handle: pool,
        }
    }
}

impl Into<ThreadPool> for TaskSpawner {
    /// Consume the TaskSpawner to get the ThreadPool.
    fn into(self: TaskSpawner) -> ThreadPool {
        self.pool_handle
    }
}

impl TaskSpawner {
    /// Create a new TaskSpawner with the default ThreadPool.
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(TaskSpawner {
            pool_handle: ThreadPool::new()?,
        })
    }
    /// Spawn a Future into the inner ThreadPool, returning the RemoteHandle for the future.
    ///
    /// For obtaining the output for a Future spawned, use `spawn_res`
    pub fn spawn<T>(&self, f: T) -> RemoteHandle<()> where T: Future<Output = ()> + Send + 'static {
        self.pool_handle.spawn_with_handle(FutureObj::new(Box::new(f))).unwrap()
    }
    /// Spawn a future into the inner ThreadPool, returning the RemoteHandle for the future.
    ///
    /// For ignoring the output of the spawned Future, use `spawn`
    pub fn spawn_res<Fut, T>(&self, f: Fut) -> RemoteHandle<T> where Fut: Future<Output = T> + Send + 'static, T: Send + 'static {
        self.pool_handle.spawn_with_handle(FutureObj::new(Box::new(f))).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::TaskSpawner;
    use std::thread::spawn;
    use futures::executor::ThreadPool;

    #[test]
    fn test_result() {
        let pool = ThreadPool::new().unwrap();
        let ev = TaskSpawner::from(pool);
        spawn(move || {
            let ev = ev.clone();
            let runner = async move {
                let res = ev.spawn_res(async {
                    5 + 5
                }).await;
                res
            };
            let t = futures::executor::block_on(runner);
            assert_eq!(t, 10);
        }).join().unwrap();
    }
}
