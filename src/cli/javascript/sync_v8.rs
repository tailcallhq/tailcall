use std::thread::ThreadId;

use mini_v8::{FromValue, ToValues};
use tokio::runtime::Handle;

use crate::ToAnyHow;

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
            rx.send(std::thread::current().id())
                .expect("failed to send thread id")
        });
        let thread_id = tx.recv().expect("failed to receive thread id");
        Self { v8, runtime, thread_id, current }
    }

    pub fn current(&self) -> Handle {
        self.current.clone()
    }

    pub async fn borrow<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mini_v8::MiniV8) -> anyhow::Result<()> + 'static,
    {
        let sync_v8 = self.clone();
        self.borrow_ret(move |_| {
            let f = SpawnBlock::new(f, sync_v8.clone());
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
        let f = SpawnBlock::new(f, self.clone());
        let (tx, mut rx) = tokio::sync::broadcast::channel::<R>(1024);
        self.runtime.spawn(async move {
            let r = f.call()?;
            tx.send(r).map_err(|e| anyhow::anyhow!(e.to_string()))
        });
        rx.recv().await.or_anyhow("failed to receive result")
    }

    pub fn as_sync_function(&self, f: mini_v8::Function) -> SyncV8Function {
        self.assert_on_v8();
        SyncV8Function { callback: f, sync_v8: self.clone() }
    }

    fn assert_on_v8(&self) -> () {
        assert_eq!(self.thread_id, std::thread::current().id())
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
        self.v8.assert_on_v8();
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
        self.sync_v8.assert_on_v8();
        self.callback
            .call(args)
            .or_anyhow("args could not be encoded to values")
    }
}

unsafe impl Send for SyncV8Function {}
unsafe impl Sync for SyncV8Function {}

impl mini_v8::ToValue for SyncV8Function {
    fn to_value(self, mv8: &mini_v8::MiniV8) -> mini_v8::Result<mini_v8::Value> {
        self.callback.to_value(mv8)
    }
}
