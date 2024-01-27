#[cfg(feature = "js")]
use std::pin::Pin;
#[cfg(feature = "js")]
use std::sync::Arc;

#[cfg(feature = "js")]
use futures_util::future::join_all;
#[cfg(feature = "js")]
use futures_util::Future;
#[cfg(feature = "js")]
use hyper::body::Bytes;

#[cfg(feature = "js")]
use crate::channel::{Command, Event, JsResponse};
#[cfg(feature = "js")]
use crate::http::Response;
#[cfg(feature = "js")]
use crate::HttpIO;
#[cfg(feature = "js")]
use crate::ScriptIO;
#[cfg(feature = "js")]
#[derive(Clone)]
pub struct HttpFilter {
  client: Arc<dyn HttpIO + Send + Sync>,
  #[cfg(feature = "js")]
  script: Arc<dyn ScriptIO<Event, Command> + Send + Sync>,
}
#[cfg(feature = "js")]
impl HttpFilter {
  pub fn new(
    http: impl HttpIO + Send + Sync,
    #[cfg(feature = "js")] script: impl ScriptIO<Event, Command> + Send + Sync + 'static,
  ) -> Self {
    HttpFilter {
      client: Arc::new(http),
      #[cfg(feature = "js")]
      script: Arc::new(script),
    }
  }
  #[cfg(feature = "js")]
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
            .flat_map(|e| JsResponse::try_from(e).ok())
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
#[cfg(feature = "js")]
#[async_trait::async_trait]
impl HttpIO for HttpFilter {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<hyper::body::Bytes>> {
    let command = self.script.on_event(Event::Request((&request).try_into()?)).await?;
    self.on_command(command).await
  }
}
