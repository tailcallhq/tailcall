package tailcall.runtime.http

import zio.http.{Client, Response, URL}
import zio.{ZIO, ZLayer}

trait HttpClient {
  def request(req: Request): ZIO[Any, Throwable, Response]
}

// TODO: handle cancellation
object HttpClient {
  final case class Live(client: Client) extends HttpClient {
    def request(req: Request): ZIO[Any, Throwable, Response] =
      for {
        resp <- client.request(req.toZHttpRequest)
        status     = resp.status.code
        isRedirect = status == 301 || status == 302 || status == 307
        headers    = resp.headers.iterator.map(h => (h.key, h.value)).toMap
        redirectResp <- client.request(req.toZHttpRequest.copy(url =
          URL.fromString(String.valueOf(headers.getOrElse("Location", "")))
            .getOrElse(throw new IllegalArgumentException(s"Invalid URL"))
        )).when(isRedirect)
      } yield redirectResp.getOrElse(resp)
  }
  def live: ZLayer[Client, Nothing, HttpClient] = ZLayer.fromFunction(client => Live(client))

  def default: ZLayer[Any, Throwable, HttpClient] = Client.default >>> live
}
