package tailcall.runtime.http

import zio.http.{Client, Response}
import zio.{ZIO, ZLayer}

trait HttpClient {
  def request(req: Request): ZIO[Any, Throwable, Response]
}

// TODO: handle cancellation
object HttpClient {
  final case class Live(client: Client) extends HttpClient {
    def request(req: Request): ZIO[Any, Throwable, Response] = { client.request(req.toZHttpRequest) }
  }
  def live: ZLayer[Client, Nothing, HttpClient] = ZLayer.fromFunction(client => Live(client))

  def default: ZLayer[Any, Throwable, HttpClient] = Client.default >>> live
}
