use crate::ToAnyHow;
use mini_v8::{FromValue, ToValues};
use std::thread::ThreadId;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct SyncV8 {
    v8: mini_v8::MiniV8,
    runtime: &'static tokio::runtime::Runtime,
    thread_id: ThreadId,
    current: Handle,
}
unsafe impl Send for SyncV8 {}
unsafe impl Sync for SyncV8 {}

lazy_static::lazy_static! {
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = {
        let r = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("mini-v8")
            .build();
        match r {
            Ok(r) => r,
            Err(e) => panic!("Failed to create tokio runtime: {}", e),
        }
    };
}

impl SyncV8 {
    pub fn new() -> Self {
        let v8 = mini_v8::MiniV8::new();
        let runtime = &TOKIO_RUNTIME;
        let (rx, tx) = std::sync::mpsc::channel::<ThreadId>();
        let current = Handle::current();
        runtime.spawn(async move {
            rx.send(std::thread::current().id()).unwrap();
        });
        let thread_id = tx.recv().unwrap();
        Self { v8, runtime, thread_id, current }
    }

    pub fn current(&self) -> Handle {
        self.current.clone()
    }

    pub async fn borrow<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mini_v8::MiniV8) -> anyhow::Result<()> + 'static,
    {
        let f = SpawnBlock::new(f, self.clone());
        self.borrow_ret(|_| {
            f.call()?;
            Ok(())
        })
        .await
    }

    pub async fn borrow_ret<R, F>(&self, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mini_v8::MiniV8) -> anyhow::Result<R> + 'static,
        R: Clone + Send + 'static,
    {
        if self.on_v8_thread() {
            panic!("SyncV8::borrow_ret called from v8 thread")
        }
        let f = SpawnBlock::new(f, self.clone());
        let (tx, mut rx) = tokio::sync::broadcast::channel::<R>(1024);
        self.runtime
            .spawn(async move { tx.send(f.call().unwrap()) });
        rx.recv().await.or_anyhow("failed to receive result")
    }

    pub fn as_sync_function(&self, f: mini_v8::Function) -> SyncV8Function {
        SyncV8Function { callback: f, sync_v8: self.clone() }
    }

    fn on_v8_thread(&self) -> bool {
        self.thread_id == std::thread::current().id()
    }
}

struct SpawnBlock<R> {
    inner: Box<dyn FnOnce(&mini_v8::MiniV8) -> anyhow::Result<R> + 'static>,
    v8: SyncV8,
}

impl<R> SpawnBlock<R> {
    fn new<F>(f: F, v8: SyncV8) -> Self
    where
        F: FnOnce(&mini_v8::MiniV8) -> anyhow::Result<R> + 'static,
    {
        Self { inner: Box::new(f), v8 }
    }

    fn call(self) -> anyhow::Result<R> {
        let current_thread = std::thread::current();
        let thread_id = current_thread.id();
        if thread_id != self.v8.thread_id {
            panic!("SpawnBlock called from wrong thread");
        }
        let v8 = self.v8.clone();
        (self.inner)(&v8.v8)
    }
}

unsafe impl<R> Send for SpawnBlock<R> {}
unsafe impl<R> Sync for SpawnBlock<R> {}

#[derive(Clone)]
pub struct SyncV8Function {
    callback: mini_v8::Function,
    sync_v8: SyncV8,
}

impl SyncV8Function {
    pub fn call<R: FromValue + Clone + 'static>(
        &self,
        args: impl ToValues + 'static,
    ) -> anyhow::Result<R> {
        if self.sync_v8.on_v8_thread() {
            self.callback
                .call(args)
                .or_anyhow("args could not be encoded to values")
        } else {
            panic!("SyncV8Function::call called from a non-v8 thread")
        }
    }
}

unsafe impl Send for SyncV8Function {}
unsafe impl Sync for SyncV8Function {}

impl mini_v8::ToValue for SyncV8Function {
    fn to_value(self, mv8: &mini_v8::MiniV8) -> mini_v8::Result<mini_v8::Value> {
        self.callback.to_value(mv8)
    }
}
