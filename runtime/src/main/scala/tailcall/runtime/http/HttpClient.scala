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
  def default: ZLayer[Any, Throwable, HttpClient] = Client.default >>> live

  def cachedDefault(cacheSize: Int): ZLayer[Any, Throwable, HttpClient] =
    HttpCache.live(cacheSize) ++ Client.default >>> cached
  def cached: ZLayer[HttpCache with Client, Nothing, Live]              =
    ZLayer(for {
      client <- ZIO.service[Client]
      cache  <- ZIO.service[HttpCache]
      _      <- cache.init(Lookup(a => client.request(a.toZHttpRequest)))
    } yield Live(client, Option(cache)))

  def live: ZLayer[Client, Nothing, HttpClient] = ZLayer.fromFunction(Live(_, None))

  final case class Live(client: Client, cache: Option[HttpCache]) extends HttpClient {
    def request(req: Request): ZIO[Any, Throwable, Response] = {
      ZIO.logSpan(s"${req.method} ${req.url}") {
        for {
          res              <-
            if (cache.nonEmpty) { cache.get.get(req) }
            else for {
              res  <- client.request(req.toZHttpRequest)
              body <- res.body.asString(StandardCharsets.UTF_8)
              _    <- ZIO.logDebug(s"code: ${res.status.code}")
              _    <- ZIO.logDebug(s"body: ${body}")
            } yield res
          redirectResponse <- if (isRedirect(res.status)) redirect(req, res) else ZIO.succeed(res)
        } yield redirectResponse
      }
    }

    private def isRedirect(status: Status) = { status.code == 301 || status.code == 302 || status.code == 307 }

    private def redirect(req: Request, res: Response): ZIO[Any, Throwable, Response] =
      for {
        location <- ZIO.fromOption(res.headers.get("Location")) <> ZIO.fail(new RuntimeException("No Location header"))
        res      <- request(req.copy(url = String.valueOf(location)))
      } yield res
  }
}
