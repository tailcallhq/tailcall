package tailcall.runtime.http

import tailcall.runtime.service.HttpCache
import zio.cache.Lookup
import zio.http.model.Status
import zio.http.{Client, Response}
import zio.{ZIO, ZLayer}

import java.nio.charset.StandardCharsets

trait HttpClient {
  def allowedHeaders: Set[String]
  def request(req: Request): ZIO[Any, Throwable, Response]
}

// TODO: handle cancellation
object HttpClient {
  def cached(allowedHeaders: Set[String]): ZLayer[HttpCache with Client, Nothing, Live] =
    ZLayer(for {
      client <- ZIO.service[Client]
      cache  <- ZIO.service[HttpCache]
      _      <- cache.init(Lookup(a => client.request(a.toZHttpRequest)))
    } yield Live(client, Option(cache), allowedHeaders))

  def cachedDefault(cacheSize: Option[Int], allowedHeaders: Set[String]): ZLayer[Any, Throwable, HttpClient] =
    cacheSize match {
      case Some(size) => HttpCache.live(size) ++ Client.default >>> cached(allowedHeaders)
      case None       => Client.default >>> live(allowedHeaders)
    }

  def default: ZLayer[Any, Throwable, HttpClient] = Client.default >>> live(Set.empty)

  def live(allowedHeaders: Set[String]): ZLayer[Client, Nothing, HttpClient] =
    ZLayer.fromFunction(Live(_, None, allowedHeaders))

  final case class Live(client: Client, cache: Option[HttpCache], allowedHeaders: Set[String]) extends HttpClient {
    def request(req: Request): ZIO[Any, Throwable, Response] = {
      for {
        res              <-
          if (cache.nonEmpty) { cache.get.get(req) <* ZIO.logInfo("cache hit") }
          else {
            for {
              res <- client.request(req.toZHttpRequest)
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
        req      <- ZIO.attempt(req.unsafeRedirect(String.valueOf(location)))
        res      <- request(req)
      } yield res
  }
}
