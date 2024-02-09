use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use deno_core::{extension, op2, OpState};
use tokio::sync::{mpsc, oneshot};

use crate::cli::javascript::JsRequest;

use super::JsResponse;

#[op2(async)]
async fn op_sleep(ms: u32) {
    tokio::time::sleep(Duration::from_millis(ms.into())).await;
}

#[op2(async)]
#[serde]
async fn op_fetch(state: Rc<RefCell<OpState>>, #[string] url: String) -> Result<JsResponse> {
    let (tx, rx) = oneshot::channel::<JsResponse>();
    let rx = {
        let state = state.borrow();
        let client =
            state.borrow::<mpsc::UnboundedSender<(oneshot::Sender<JsResponse>, JsRequest)>>();

        // TODO: extend options
        let request = JsRequest {
            url,
            method: "GET".to_string(),
            headers: Default::default(),
            body: Default::default(),
        };

        client.send((tx, request)).unwrap();

        rx
    };

    Ok(rx.await?)
}

extension!(console, js = ["src/cli/javascript/shim/console.js",]);
extension!(
    timer_promises,
    ops = [op_sleep],
    js = ["src/cli/javascript/shim/timer_promises.js"]
);
extension!(
    fetch,
    ops = [op_fetch],
    js = ["src/cli/javascript/shim/fetch.js"]
);
