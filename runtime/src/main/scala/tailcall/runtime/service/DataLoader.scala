package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import zio._
import zio.http.{Request => ZRequest}

final case class DataLoader[-R, E, A, B](map: Ref[Map[A, Promise[E, B]]], resolver: A => ZIO[R, E, B]) {
  self =>

  def load(a: A): ZIO[R, E, B] = {
    for {
      newPromise <- Promise.make[E, B]
      result     <- map.modify(map =>
        map.get(a) match {
          case Some(promise) => ((false, promise), map)
          case None          => ((true, newPromise), map + (a -> newPromise))
        }
      )
      cond = result._1
      promise = result._2
      _ <- resolver(a).flatMap(promise.succeed(_)).when(cond).catchAll(e =>
        for {
          _ <- promise.fail(e)
          _ <- map.update(_ - a)
        } yield ()
      )
      b <- promise.await
    } yield b
  }

  def widenError[E1](implicit ev: E <:< E1): DataLoader[R, E1, A, B] = self.asInstanceOf[DataLoader[R, E1, A, B]]
}

object DataLoader {
  type HttpDataLoader = DataLoader[Any, Throwable, Request, Chunk[Byte]]
  // TODO: make this configurable
  val allowedHeaders = Set("authorization", "cookie")

  def http: ZLayer[HttpClient, Nothing, HttpDataLoader] = http(None)

  def http(req: Option[ZRequest] = None): ZLayer[HttpClient, Nothing, HttpDataLoader] =
    ZLayer {
      ZIO.service[HttpClient].flatMap { client =>
        DataLoader.make[Request] { request =>
          val finalHeaders = request.headers ++ getForwardedHeaders(req)
          for {
            response <- client.request(request.copy(headers = finalHeaders))
            _ <- ValidationError.StatusCodeError(response.status.code, request.url).when(response.status.code >= 400)
            chunk <- response.body.asChunk
          } yield chunk

        }
      }
    }

  def load(request: Request): ZIO[HttpContext, Throwable, Chunk[Byte]] =
    ZIO.serviceWithZIO[HttpContext](_.dataLoader.load(request))

  def make[A]: PartiallyAppliedDataLoader[A] = new PartiallyAppliedDataLoader(())

  private def getForwardedHeaders(req: Option[ZRequest]): Map[String, String] = {
    req.map(_.headers.toList.filter(x => allowedHeaders.contains(String.valueOf(x.key).toLowerCase())))
      .getOrElse(List.empty).map(header => (String.valueOf(header.key), String.valueOf(header.value))).toMap
  }

  final class PartiallyAppliedDataLoader[A](val unit: Unit) {
    def apply[R, E, B](f: A => ZIO[R, E, B]): ZIO[Any, Nothing, DataLoader[R, E, A, B]] =
      for { cache <- Ref.make(Map.empty[A, Promise[E, B]]) } yield DataLoader(cache, f)
  }
}
