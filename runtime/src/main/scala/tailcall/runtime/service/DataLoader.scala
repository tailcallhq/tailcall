package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import zio._

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
      _     <- resolver(a).flatMap(promise.succeed(_)).when(cond)
      chunk <- promise.await
    } yield chunk
  }
}

object DataLoader {
  type HttpDataLoader = DataLoader[Any, Throwable, Request, Chunk[Byte]]

  def load(request: Request): ZIO[HttpDataLoader, Throwable, Chunk[Byte]] =
    ZIO.serviceWithZIO[HttpDataLoader](_.load(request))

  def http: ZLayer[HttpClient, Nothing, HttpDataLoader] =
    ZLayer {
      for {
        client <- ZIO.service[HttpClient]
        map    <- Ref.make(Map.empty[Request, Promise[Throwable, Chunk[Byte]]])
        resolver = (request: Request) =>
          client.request(request).flatMap { x =>
            if (x.status.code >= 400) ZIO.fail(new Throwable(s"HTTP Error: ${x.status.code}")) else { x.body.asChunk }
          }
      } yield DataLoader(map, resolver)
    }
}
