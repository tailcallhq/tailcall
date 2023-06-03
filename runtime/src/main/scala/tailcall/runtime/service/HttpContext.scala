package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import zio._
import zio.http.Headers
import zio.http.{Request => ZRequest}

trait HttpContext {
  def dataLoader: DataLoader[Any, Throwable, Request, Chunk[Byte]]
  def headers: Headers
}

object HttpContext {
  def default: ZLayer[Any, Throwable, HttpContext]                          = HttpClient.default >>> live(None)
  def live(req: Option[ZRequest]): ZLayer[HttpClient, Nothing, HttpContext] =
    DataLoader.http(req) >>> ZLayer {
      for {
        dataLoader <- ZIO.service[DataLoader[Any, Throwable, Request, Chunk[Byte]]]
      } yield Live(dataLoader, req.map(_.headers).getOrElse(Headers.empty))
    }

  final case class Live(dataLoader: DataLoader[Any, Throwable, Request, Chunk[Byte]], headers: Headers)
      extends HttpContext {}
}
