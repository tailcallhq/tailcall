use std::pin::Pin;
use std::sync::Arc;

use futures_util::future::join_all;
use futures_util::Future;
use hyper::body::Bytes;

use crate::channel::{Command, Event, JsResponse};
use crate::http::Response;
use crate::{HttpIO, ScriptIO};

#[derive(Clone)]
pub struct HttpFilter {
  client: Arc<dyn HttpIO + Send + Sync>,
  script: Arc<dyn ScriptIO<Event, Command> + Send + Sync>,
}

impl HttpFilter {
  pub fn new(http: impl HttpIO + Send + Sync, script: impl ScriptIO<Event, Command> + Send + Sync + 'static) -> Self {
    HttpFilter { client: Arc::new(http), script: Arc::new(script) }
  }

  fn on_command<'a>(
    &'a self,
    command: Command,
  ) -> Pin<Box<dyn Future<Output = anyhow::Result<Response<hyper::body::Bytes>>> + Send + 'a>> {
    Box::pin(async move {
      match command {
        Command::Request(requests) => {
          let requests = requests.into_iter().flat_map(|req| {
            let req = req.try_into().ok()?;
            Some(self.client.execute(req))
          });
          let responses = join_all(requests)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .iter()
            .flat_map(|e| Some(JsResponse::try_from(e).ok()?))
            .collect::<Vec<_>>();
          let command = self.script.on_event(Event::Response(responses)).await?;
          Ok(self.on_command(command).await?)
        }
        Command::Response(response) => {
          let res: anyhow::Result<Response<Bytes>> = response.try_into();
          res
        }
        Command::Continue(request) => {
          let res = self.client.execute(request.try_into()?).await?;
          Ok(res)
        }
      }
    })
  }
}

#[async_trait::async_trait]
impl HttpIO for HttpFilter {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<hyper::body::Bytes>> {
    let command = self.script.on_event(Event::Request((&request).try_into()?)).await?;
    self.on_command(command).await
  }
}
