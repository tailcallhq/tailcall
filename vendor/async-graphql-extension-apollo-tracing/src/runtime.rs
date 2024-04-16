cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub use tokio::task::JoinHandle;
        pub fn spawn(f: impl std::future::Future<Output = ()> + Send + 'static) -> JoinHandle<()> {
            tokio::spawn(f)
        }

        pub fn abort(handle: &JoinHandle<()>) {
            handle.abort();
        }

        pub struct Instant(tokio::time::Instant);
        impl Instant {
            pub fn now() -> Instant {
                Instant(tokio::time::Instant::now())
            }

            pub fn elapsed(&self) -> std::time::Duration {
                self.0.elapsed()
            }
        }
    } else {
        pub struct JoinHandle<T: Send + 'static>(std::marker::PhantomData<T>);

        pub fn spawn(f: impl futures::future::Future<Output = ()> + 'static) -> JoinHandle<()> {
            wasm_bindgen_futures::spawn_local(f);
            JoinHandle(std::marker::PhantomData)
        }

        pub fn abort(_handle: &JoinHandle<()>) {}

        pub struct Instant(std::time::Instant);
        impl Instant {
            pub fn now() -> Instant {
                Instant(std::time::Instant::now())
            }

            pub fn elapsed(&self) -> std::time::Duration {
                self.0.elapsed()
            }
        }
    }
}
