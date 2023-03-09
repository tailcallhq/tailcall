package tailcall.runtime.http

import io.netty.handler.codec.http._
import zio.http._
import zio.http.model.{Headers => zHeaders, Method => zMethod, Version}
import zio.{Chunk, Task, ZIO, ZLayer}

trait HttpClient {
  def request(req: HttpRequest): HttpClient.AsyncHandler
}

// TODO: handle cancellation
object HttpClient {

  type Response     = (Int, Map[CharSequence, CharSequence], Task[Array[Byte]])
  type AsyncHandler = ZIO[Any, Throwable, Response]
  final case class Live(client: Client) extends HttpClient {
    def request(req: HttpRequest): AsyncHandler = {
      client.request(Request(
        method = zMethod.fromHttpMethod(req.method),
        url = URL.fromString(req.uri).getOrElse(throw new IllegalArgumentException(s"Invalid URL: ${req.uri}")),
        headers = zHeaders.make(req.headers),
        body = req match {
          case request: FullHttpRequest => Body.fromChunk(Chunk.fromArray(request.content.array()))
          case _: DefaultHttpRequest    => Body.fromChunk(Chunk.empty)
        },
        version = Version.`HTTP/1.1`,
        remoteAddress = None
      )).map(response =>
        (response.status.code, response.headers.map(header => header.key -> header.value).toMap, response.body.asArray)
      )
    }
  }
  def live: ZLayer[Client, Throwable, HttpClient] = ZLayer.fromFunction(client => Live(client))
}
