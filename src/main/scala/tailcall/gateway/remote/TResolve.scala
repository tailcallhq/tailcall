package tailcall.gateway.remote

import zio.schema.Schema

final case class TResolve[-R, +E, +A](remote: R ~> Either[E, A]) {
  self =>

  def provide(r: Remote[R]): TResolve[Any, E, A] =
    TResolve(Lambda.fromFunction[Any, Either[E, A]](_ => self.remote(r)))

  def map[B](f: Remote[A] => Remote[B]): TResolve[R, E, B] =
    self.flatMap(a => TResolve.succeed(f(a)))

  def flatMap[R1 <: R, E1 >: E, B](
    f: Remote[A] => TResolve[R1, E1, B]
  ): TResolve[R1, E1, B] =
    TResolve.collect[R1](r =>
      self.remote(r).fold(e => Lambda.fromEither(Left(e)), a => f(a).remote(r))
    )
}

object TResolve {
  def succeed[A](a: Remote[A]): TResolve[Any, Nothing, A] =
    fromEither(Lambda.fromEither(Right(a)))

  def fail[E](e: Remote[E]): TResolve[Any, E, Nothing] =
    fromEither(Lambda.fromEither(Left(e)))

  def fromEither[E, A](e: Remote[Either[E, A]]): TResolve[Any, E, A] =
    TResolve.collect[Any](_ => e)

  def collect[R]: PartialCollect[R] = new PartialCollect[R](())

  final class PartialCollect[R](private val dummy: Unit) extends AnyVal {
    def apply[E, A](f: Remote[R] => Remote[Either[E, A]]): TResolve[R, E, A] =
      TResolve(Lambda.fromFunction[R, Either[E, A]](f))
  }

  implicit def schema[R, E, A]: Schema[TResolve[R, E, A]] =
    Schema[R ~> Either[E, A]].transform(TResolve(_), _.remote)
}
