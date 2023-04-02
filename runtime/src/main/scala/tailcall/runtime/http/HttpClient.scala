package tailcall.runtime.http

import zio.http.model.Status
import zio.http.{Client, Response}
import zio.{ZIO, ZLayer}

trait HttpClient {
  def request(req: Request): ZIO[Any, Throwable, Response]
}

// TODO: handle cancellation
object HttpClient {
  def default: ZLayer[Any, Throwable, HttpClient] = Client.default >>> live

  def live: ZLayer[Client, Nothing, HttpClient] = ZLayer.fromFunction(client => Live(client))

  final case class Live(client: Client) extends HttpClient {
    def request(req: Request): ZIO[Any, Throwable, Response] =
      for {
        resp             <- client.request(req.toZHttpRequest)
        redirectResponse <- if (isRedirect(resp.status)) redirect(req) else ZIO.succeed(resp)
      } yield redirectResponse

    private def isRedirect(status: Status) = { status.code == 301 || status.code == 302 || status.code == 307 }

    private def redirect(req: Request): ZIO[Any, Throwable, Response] =
      for {
        location <- ZIO.fromOption(req.headers.get("Location")) <> ZIO.fail(new RuntimeException("No Location header"))
        res      <- request(req.copy(url = String.valueOf(location)))
      } yield res
  }
}
