package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import tailcall.runtime.service.HttpContext.State
import zio._
import zio.http.model.Headers
import zio.http.{Request => ZRequest, Response}

trait HttpContext {
  def dataLoader: DataLoader[Any, Throwable, Request, Response]
  def requestHeaders: Headers
  final def getState: UIO[State] = update(identity)
  def update(state: State => State): UIO[State]
}

object HttpContext {
  def default: ZLayer[Any, Throwable, HttpContext]                = HttpClient.default >>> live(None)
  def update(f: State => State): ZIO[HttpContext, Nothing, State] = ZIO.serviceWithZIO(_.update(f))
  def getState: ZIO[HttpContext, Nothing, State]                  = ZIO.serviceWithZIO(_.getState)

  def live(req: Option[ZRequest]): ZLayer[HttpClient, Nothing, HttpContext] =
    DataLoader.http(req) >>> ZLayer {
      for {
        ref        <- Ref.make(State.empty)
        dataLoader <- ZIO.service[DataLoader[Any, Throwable, Request, Response]]
      } yield Live(dataLoader, req.map(_.headers).getOrElse(Headers.empty), ref)
    }

  final case class State(cacheMaxAge: Duration) {
    def withCacheMaxAge(maxAge: Duration): State =
      copy(cacheMaxAge = Math.min(cacheMaxAge.toMillis, maxAge.toMillis) millis)
  }
  object State                                  {
    def empty: State = State(0 second)
  }

  final case class Live(
    dataLoader: DataLoader[Any, Throwable, Request, Response],
    requestHeaders: Headers,
    ref: Ref[State],
  ) extends HttpContext {
    override def update(state: State => State): UIO[State] = ref.updateAndGet(state)
  }
}
