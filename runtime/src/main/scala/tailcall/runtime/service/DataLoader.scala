package tailcall.runtime.service

import tailcall.runtime.http.{HttpClient, Request}
import zio._
import zio.http.{Request => ZRequest, Response}

final case class DataLoader[R, E, A, B](
  ref: Ref[DataLoader.State[E, A, B]],
  resolver: DataLoader.Resolver[R, E, A, B],
) {
  self =>

  /**
   * Collects the requests and returns an IO wrapped in a
   * UIO. The inner IO resolves when the dispatch is
   * completed.
   */
  def collect(seq: A*): UIO[Chunk[ZIO[Any, E, B]]] = ZIO.foreach(seq)(collect).map(Chunk.from)

  def collect(a: A): UIO[ZIO[Any, E, B]] = insert(a).map(_._2.await)

  /**
   * Provides manual control over when the resolver should
   * be called. When called concurrently it is possible that
   * the data loader issues multiple requests.
   */
  def dispatch: ZIO[R, Nothing, Unit] = {
    for {
      state   <- ref.get
      pending <- state.pending
      keys   = pending.map(_._1)
      values = pending.map(_._2)
      _ <- resolver(keys).flatMap { bChunks =>
        ZIO.foreachDiscard(values.zip(bChunks)) { case (promise, b) => promise.succeed(b) }
      }.catchAll { error =>
        for {
          _ <- ref.update(state => state.drop(keys))
          _ <- ZIO.foreachDiscard(state.map.values)(_.fail(error))
        } yield ()
      }
    } yield ()
  }

  /**
   * Load a value from the data loader and caches the
   * response from the resolver for the data-loader's life
   * time.
   */
  def load(a: A): ZIO[R, E, B] =
    for {
      state    <- insert(a)
      _        <- resolver(a).intoPromise(state._2).when(state._1)
      response <- state._2.await
    } yield response

  private def insert(a: A): ZIO[Any, Nothing, (Boolean, Promise[E, B])] =
    for {
      nPromise <- Promise.make[E, B]
      result   <- ref.modify { state =>
        state.get(a) match {
          case Some(promise) => ((false, promise), state)
          case None          => ((true, nPromise), state.add(a, nPromise))
        }
      }
    } yield result
}

object DataLoader {
  type HttpDataLoader = DataLoader[Any, Throwable, Request, Response]
  // TODO: make this configurable
  private val allowedHeaders: Set[String] = Set("authorization", "cookie", "apikey")

  def dispatch: ZIO[HttpContext, Throwable, Unit] = ZIO.serviceWithZIO[HttpContext](_.dataLoader.dispatch)

  def http: ZLayer[HttpClient, Nothing, HttpDataLoader] = http(None)

  def http(req: Option[ZRequest] = None): ZLayer[HttpClient, Nothing, HttpDataLoader] =
    ZLayer {
      ZIO.serviceWithZIO[HttpClient] { client =>
        DataLoader.one[Request] { request =>
          val finalHeaders = request.headers ++ getForwardedHeaders(req)
          for {
            response <- client.request(request.copy(headers = finalHeaders))
            _ <- ValidationError.StatusCodeError(response.status.code, request.url).when(response.status.code >= 400)

          } yield response
        }
      }
    }

  def httpCollect(requests: Chunk[Request]): ZIO[HttpContext, Throwable, Chunk[ZIO[Any, Throwable, Response]]] =
    ZIO.serviceWithZIO[HttpContext](_.dataLoader.collect(requests: _*))

  def httpLoad(request: Request): ZIO[HttpContext, Throwable, Response] =
    ZIO.serviceWithZIO[HttpContext](_.dataLoader.load(request))

  def many[A]: PartiallyAppliedDataLoaderMany[A] = new PartiallyAppliedDataLoaderMany(())

  def one[A]: PartiallyAppliedDataLoaderOne[A] = new PartiallyAppliedDataLoaderOne(())

  private def getForwardedHeaders(req: Option[ZRequest]): Map[String, String] = {
    req.map(_.headers.toList.filter(x => allowedHeaders.contains(String.valueOf(x.key).toLowerCase())))
      .getOrElse(List.empty).map(header => (String.valueOf(header.key), String.valueOf(header.value))).toMap
  }

  sealed trait Resolver[R, E, A, B] {
    self =>
    def apply(a: A): ZIO[R, E, B] =
      self match {
        case Resolver.One(f)  => f(a)
        case Resolver.Many(f) => f(Chunk(a)).map(_.head)
      }

    def apply(a: Chunk[A]): ZIO[R, E, Chunk[B]] =
      self match {
        case Resolver.One(f)  => ZIO.foreachPar(a)(f)
        case Resolver.Many(f) => f(a)
      }
  }

  final case class State[E, A, B](map: Map[A, Promise[E, B]] = Map.empty[A, Promise[E, B]]) {
    self =>
    def add(a: A, promise: Promise[E, B]): State[E, A, B] = copy(map = map + (a -> promise))

    def drop(a: A): State[E, A, B] = copy(map = map - a)

    def drop(a: Chunk[A]): State[E, A, B] = copy(map = map.filterNot(i => a.contains(i._1)))

    def get(a: A): Option[Promise[E, B]] = map.get(a)

    def pending: UIO[Chunk[(A, Promise[E, B])]] =
      ZIO.foreach(Chunk.from(map)) { case (key, promise) =>
        promise.isDone.map(done => if (done) Chunk.empty else Chunk.single(key -> promise))
      }.map(_.flatten)
  }

  final class PartiallyAppliedDataLoaderOne[A](val unit: Unit) {
    def apply[R, E, B](f: A => ZIO[R, E, B]): ZIO[Any, Nothing, DataLoader[R, E, A, B]] =
      for { ref <- Ref.make(State[E, A, B]()) } yield DataLoader(ref, Resolver.One(f))
  }

  final class PartiallyAppliedDataLoaderMany[A](val unit: Unit) {
    def apply[R, E, B](f: Chunk[A] => ZIO[R, E, Chunk[B]]): ZIO[Any, Nothing, DataLoader[R, E, A, B]] =
      for { ref <- Ref.make(State[E, A, B]()) } yield DataLoader(ref, Resolver.Many(f))
  }

  object Resolver {
    final case class One[R, E, A, B](f: A => ZIO[R, E, B])                extends Resolver[R, E, A, B]
    final case class Many[R, E, A, B](f: Chunk[A] => ZIO[R, E, Chunk[B]]) extends Resolver[R, E, A, B]
  }
}
