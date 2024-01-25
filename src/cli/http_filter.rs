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
          let responses = join_all(requests.into_iter().map(|request| self.client.execute(request.into())))
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
          let command = self
            .script
            .on_event(Event::Response(
              responses.iter().map(JsResponse::from).collect::<Vec<_>>(),
            ))
            .await?;
          Ok(self.on_command(command).await?)
        }
        Command::Response(response) => {
          let res: Response<Bytes> = response.into();
          Ok(res)
        }
        Command::Continue(request) => {
          let res = self.client.execute(request.into()).await?;
          Ok(res)
        }
      }
    })
  }
}

#[async_trait::async_trait]
impl HttpIO for HttpFilter {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<hyper::body::Bytes>> {
    let command = self.script.on_event(Event::Request((&request).into())).await?;
    self.on_command(command).await
  }
}
