package tailcall.runtime.http

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

  def live: ZLayer[Client, Nothing, HttpClient] = ZLayer.fromFunction(client => Live(client))

  final case class Live(client: Client) extends HttpClient {
    def request(req: Request): ZIO[Any, Throwable, Response] = {
      ZIO.logSpan(s"${req.method} ${req.url}") {
        for {
          res              <- client.request(req.toZHttpRequest)
          body             <- res.body.asString(StandardCharsets.UTF_8)
          _                <- ZIO.log(s"code: ${res.status.code} body: ${body}")
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
