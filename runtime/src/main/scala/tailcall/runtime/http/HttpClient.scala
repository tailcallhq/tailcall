package tailcall.runtime.http

import tailcall.runtime.service.HttpCache
import zio.cache.Lookup
import zio.http.model.Status
import zio.http.{Client, Response}
import zio.{ZIO, ZLayer}

import java.nio.charset.StandardCharsets

trait HttpClient {
  def request(req: Request): ZIO[Any, Throwable, Response]
}

// TODO: handle cancellation
object HttpClient {
  def cached: ZLayer[HttpCache with Client, Nothing, Live] =
    ZLayer(for {
      client <- ZIO.service[Client]
      cache  <- ZIO.service[HttpCache]
      _      <- cache.init(Lookup(a => client.request(a.toZHttpRequest)))
    } yield Live(client, Option(cache)))

  def cachedDefault(cacheSize: Option[Int]): ZLayer[Any, Throwable, HttpClient] =
    cacheSize match {
      case Some(size) => HttpCache.live(size) ++ Client.default >>> cached
      case None       => Client.default >>> live
    }

  def default: ZLayer[Any, Throwable, HttpClient] = Client.default >>> live

  def live: ZLayer[Client, Nothing, HttpClient] = ZLayer.fromFunction(Live(_, None))

  final case class Live(client: Client, cache: Option[HttpCache]) extends HttpClient {
    def request(req: Request): ZIO[Any, Throwable, Response] = {
      for {
        res              <-
          if (cache.nonEmpty) { cache.get.get(req) <* ZIO.logInfo("cache hit") }
          else {
            for {
              res <- client.request(req.toZHttpRequest.withUserAgent(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"
              ))
              _   <- ZIO.logInfo(s"cache miss")
            } yield res
          }
        body             <- res.body.asString(StandardCharsets.UTF_8)
        _                <- ZIO.logInfo(s"${res.status.code}")
        _                <- ZIO.logDebug(s"body: ${body}")
        redirectResponse <- if (isRedirect(res.status)) redirect(req, res) else { ZIO.succeed(res) }
      } yield redirectResponse
    }

    private def isRedirect(status: Status) = { status.code == 301 || status.code == 302 || status.code == 307 }

    private def redirect(req: Request, res: Response): ZIO[Any, Throwable, Response] =
      for {
        location <- ZIO.fromOption(res.headers.get("Location")) <> ZIO.fail(new RuntimeException("No Location header"))
        res      <- request(req.copy(url = String.valueOf(location)))
      } yield res
  }
}
