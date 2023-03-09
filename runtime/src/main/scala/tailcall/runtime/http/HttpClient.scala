package tailcall.runtime.http

import zio.http.Client
import zio.{Task, ZIO, ZLayer}

trait HttpClient {
  def request(req: Request): HttpClient.AsyncHandler
}

// TODO: handle cancellation
object HttpClient {

  type Response     = (Int, Map[CharSequence, CharSequence], Task[Array[Byte]])
  type AsyncHandler = ZIO[Any, Throwable, Response]
  final case class Live(client: Client) extends HttpClient {
    def request(req: Request): AsyncHandler = {
      client.request(req.toZHttpRequest).map(response =>
        (response.status.code, response.headers.map(header => header.key -> header.value).toMap, response.body.asArray)
      )
    }
  }
  def live: ZLayer[Client, Throwable, HttpClient] = ZLayer.fromFunction(client => Live(client))
}
