use futures_util::stream::FuturesUnordered;
use futures_util::{Future, StreamExt};

#[derive(Clone, Debug)]
pub enum Concurrency {
  Parallel,
  Sequential,
}

impl Concurrency {
  pub async fn fold<F, A, B>(
    &self,
    iter: impl Iterator<Item = F>,
    acc: B,
    f: impl Fn(B, A) -> anyhow::Result<B>,
  ) -> anyhow::Result<B>
  where
    F: Future<Output = A>,
  {
    match self {
      Concurrency::Sequential => {
        let mut output = acc;
        for future in iter.into_iter() {
          output = f(output, future.await)?;
        }
        Ok(output)
      }
      Concurrency::Parallel => {
        let mut futures: FuturesUnordered<_> = iter.into_iter().collect();
        let mut output = acc;
        while let Some(result) = futures.next().await {
          output = f(output, result)?;
        }
        Ok(output)
      }
    }
  }

  pub async fn foreach<F, A, B>(&self, iter: impl Iterator<Item = F>, f: impl Fn(A) -> B) -> anyhow::Result<Vec<B>>
  where
    F: Future<Output = anyhow::Result<A>>,
  {
    self
      .fold(iter, vec![], |mut acc, val| {
        acc.push(f(val?));
        Ok(acc)
      })
      .await
  }
}
