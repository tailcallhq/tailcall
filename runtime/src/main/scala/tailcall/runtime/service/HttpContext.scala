package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import zio._
import zio.http.model.Headers
import zio.http.{Response, Request => ZRequest}

trait HttpContext {
  def dataLoader: DataLoader[Any, Throwable, Request, Response]
  def requestHeaders: Headers
  final def responseHeaders: UIO[Headers] = updateResponseHeaders(identity)

  def updateResponseHeaders(headers: Headers =>  Headers): UIO[Headers]
}

object HttpContext {
  def default: ZLayer[Any, Throwable, HttpContext]                          = HttpClient.default >>> live(None)
  def live(req: Option[ZRequest]): ZLayer[HttpClient, Nothing, HttpContext] =
    DataLoader.http(req) >>> ZLayer {
      for {
        ref <- Ref.make(Headers.empty)
        dataLoader <- ZIO.service[DataLoader[Any, Throwable, Request, Response]]
      } yield Live(dataLoader, req.map(_.headers).getOrElse(Headers.empty), ref)
    }

  final case class Live(dataLoader: DataLoader[Any, Throwable, Request, Response], requestHeaders: Headers, ref: Ref[Headers])
      extends HttpContext {
    override def updateResponseHeaders(f: Headers => Headers): UIO[Headers] = ref.updateAndGet(f)

  }
}
