package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import tailcall.runtime.internal.HttpAssertions
import zio._
import zio.http.{Request => ZRequest}

case class DataLoader[R, E, A, B](map: Ref[Map[A, Promise[E, B]]], resolver: A => ZIO[R, E, B]) {
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
      _ <- resolver(a).flatMap(promise.succeed(_)).when(cond)
      b <- promise.await
    } yield b
  }
}

object DataLoader {
  type HttpDataLoader = DataLoader[Any, Throwable, Request, Chunk[Byte]]

  def load(request: Request): ZIO[HttpDataLoader, Throwable, Chunk[Byte]]             =
    ZIO.serviceWithZIO[HttpDataLoader](_.load(request))
  // TODO: make this configurable
  val allowedHeaders                                                                  = Set("authorization", "cookie")
  def http: ZLayer[HttpClient, Nothing, HttpDataLoader]                               = http(None)
  def http(req: Option[ZRequest] = None): ZLayer[HttpClient, Nothing, HttpDataLoader] =
    ZLayer {
      for {
        client       <- ZIO.service[HttpClient]
        requestCache <- Ref.make(Map.empty[Request, Promise[Throwable, Chunk[Byte]]])
        headers  = getForwardedHeaders(req)
        resolver = (request: Request) => {
          val finalHeaders = request.headers ++ headers
          for {
            response <- client.request(request.copy(headers = finalHeaders))
            _        <- HttpAssertions.assertStatusCodeIsAbove(400, response)
            chunk    <- response.body.asChunk
          } yield chunk
        }
      } yield DataLoader(requestCache, resolver)
    }

  private def getForwardedHeaders(req: Option[ZRequest]) = {
    req.map(_.headers.toList.filter(x => allowedHeaders.contains(String.valueOf(x.key).toLowerCase())))
      .getOrElse(List.empty).map(header => (String.valueOf(header.key), String.valueOf(header.value))).toMap
  }
}
