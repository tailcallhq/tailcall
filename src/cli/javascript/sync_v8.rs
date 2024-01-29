use std::sync::Arc;
use std::thread::ThreadId;

use log::kv::ToValue;
use mini_v8::{FromValue, ToValues, Values};

use crate::ToAnyHow;

#[derive(Clone)]
pub struct SyncV8 {
    v8: mini_v8::MiniV8,
    runtime: Arc<tokio::runtime::Runtime>,
    thread_id: ThreadId,
}

impl SyncV8 {
    pub fn new() -> Self {
        let v8 = mini_v8::MiniV8::new();
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("mini-v8")
                .build()
                .unwrap(),
        );
        let (rx, tx) = std::sync::mpsc::channel::<ThreadId>();

        runtime.spawn(async move {
            rx.send(std::thread::current().id()).unwrap();
        });
        let thread_id = tx.recv().unwrap();
        Self { v8, runtime, thread_id }
    }

    pub fn borrow<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mini_v8::MiniV8) -> anyhow::Result<()> + 'static,
    {
        let runtime = self.runtime.clone();
        let f = SpawnBlock::new(f, self.clone());

        runtime.spawn(async move {
            f.call();
        });

        Ok(())
    }

    pub fn borrow_ret<R, F>(&self, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mini_v8::MiniV8) -> anyhow::Result<R> + 'static,
        R: Send + 'static,
    {
        let runtime = self.runtime.clone();
        let f = SpawnBlock::new(f, self.clone());
        let (tx, rx) = std::sync::mpsc::channel::<anyhow::Result<R>>();
        runtime.spawn(async move { tx.send(f.call()) });
        rx.recv().unwrap()
    }

    pub fn create_function<F, A, R>(&self, _f: F) -> Box<dyn Fn(A) -> anyhow::Result<R>>
    where
        F: Fn(&A) -> anyhow::Result<R> + Send + 'static,
        A: Send + 'static,
        R: Send + 'static,
    {
        todo!()
    }

    pub fn as_sync_function(&self, f: mini_v8::Function) -> SyncV8Function {
        SyncV8Function { callback: f, sync_v8: self.clone() }
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
    pub fn call<R: FromValue + 'static>(&self, args: impl ToValues + 'static) -> anyhow::Result<R> {
        let (tx, rx) = std::sync::mpsc::channel::<anyhow::Result<R>>();
        let callback = self.callback.clone();
        self.sync_v8.borrow(move |mv8| {
            let r = callback
                .call::<Values, R>(
                    args.to_values(mv8)
                        .or_anyhow("args could not be encoded to values")?,
                )
                .or_anyhow("function call failed");
            tx.send(r).unwrap();
            Ok(())
        })?;
        rx.recv().unwrap()
    }
}

unsafe impl Send for SyncV8Function {}
unsafe impl Sync for SyncV8Function {}

impl mini_v8::ToValue for SyncV8Function {
    fn to_value(self, mv8: &mini_v8::MiniV8) -> mini_v8::Result<mini_v8::Value> {
        self.callback.to_value(mv8)
    }
}
