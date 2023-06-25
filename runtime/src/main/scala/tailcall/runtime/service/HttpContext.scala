package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import zio._
import zio.http.model.Headers
import zio.http.{Request => ZRequest, Response}

trait HttpContext {
  def dataLoader: DataLoader[Any, Throwable, Request, Response]
  def requestHeaders: Headers

  // final def responseHeaders: UIO[Headers] = updateResponseHeaders(identity)

  final def getState: UIO[Option[Duration]] = updateState(identity).map(_.value)

  def updateState(
    state: HttpContext.State[Option[Duration]] => HttpContext.State[Option[Duration]]
  ): UIO[HttpContext.State[Option[Duration]]]
}

object HttpContext {
  def default: ZLayer[Any, Throwable, HttpContext] = HttpClient.default >>> live(None)

  def getState: ZIO[HttpContext, Nothing, Option[Duration]] = ZIO.serviceWithZIO(_.getState)

  def live(req: Option[ZRequest]): ZLayer[HttpClient, Nothing, HttpContext] =
    DataLoader.http(req) >>> ZLayer {
      for {
        ref        <- Ref.make(State[Option[Duration]](None))
        dataLoader <- ZIO.service[DataLoader[Any, Throwable, Request, Response]]
      } yield Live(dataLoader, req.map(_.headers).getOrElse(Headers.empty), ref)
    }

  final case class State[+A](value: A)
  final case class Live(
    dataLoader: DataLoader[Any, Throwable, Request, Response],
    requestHeaders: Headers,
    ref: Ref[State[Option[Duration]]],
  ) extends HttpContext {
    override def updateState(state: State[Option[Duration]] => State[Option[Duration]]): UIO[State[Option[Duration]]] =
      ref.updateAndGet(state)
  }
}
