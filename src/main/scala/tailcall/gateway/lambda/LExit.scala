package tailcall.gateway.lambda

import zio._

final case class LExit[-R, +E, -A, +B](run: A => ZIO[R, E, B])
    extends (A => ZIO[R, E, B]) {
  self =>

  def apply(a: A): ZIO[R, E, B] = run(a)

  def pipe[R1 <: R, E1 >: E, A1 >: B, B1](
    other: LExit[R1, E1, A1, B1]
  ): LExit[R1, E1, A, B1] = LExit(a => run(a).flatMap(b => other.run(b)))

  def >>>[R1 <: R, E1 >: E, A1 >: B, B1](
    other: LExit[R1, E1, A1, B1]
  ): LExit[R1, E1, A, B1] = self.pipe(other)

  def flatMap[R1 <: R, E1 >: E, A1 <: A, B1](
    f: B => LExit[R1, E1, A1, B1]
  ): LExit[R1, E1, A1, B1] = LExit(a => run(a).flatMap(b => f(b).run(a)))

  def map[C](f: B => C): LExit[R, E, A, C] = LExit(a => run(a).map(f))

  def provideInput(a: A): LExit[R, E, Any, B] = LExit(_ => run(a))

  def mapError[E1](f: E => E1): LExit[R, E1, A, B] =
    LExit(a => run(a).mapError(f))

  def debug(msg: String): LExit[R, E, A, B] = LExit(a => run(a).debug(msg))
}

object LExit {
  def succeed[A](a: A): LExit[Any, Nothing, Any, A] = LExit(_ => ZIO.succeed(a))

  def fail[E](e: E): LExit[Any, E, Any, Nothing] = LExit(_ => ZIO.fail(e))

  def fromEither[E, A](either: Either[E, A]): LExit[Any, E, Any, A] =
    either.fold(fail, succeed)

  def fromZIO[R, E, B](effect: ZIO[R, E, B]): LExit[R, E, Any, B] =
    LExit(_ => effect)

  def input[A]: LExit[Any, Nothing, A, A] = LExit(ZIO.succeed(_))

  def foreach[R, E, A, I, B, Collection[+Element] <: Iterable[Element]](
    in: Collection[I]
  )(f: I => LExit[R, E, A, B])(implicit
    bf: BuildFrom[Collection[I], B, Collection[B]]
  ): LExit[R, E, A, Collection[B]] =
    LExit(a => ZIO.foreach(in)(i => f(i).run(a)))

  def filter[R, E, A, I, Collection[+Element] <: Iterable[Element]](
    as: Collection[I]
  )(f: I => LExit[R, E, A, Boolean])(implicit
    bf: BuildFrom[Collection[I], I, Collection[I]]
  ): LExit[R, E, A, Collection[I]] =
    LExit(a => ZIO.filter(as)(i => f(i).run(a)))

  def none: LExit[Any, Nothing, Any, Option[Nothing]] =
    LExit(_ => ZIO.succeed(None))

  def attempt[A](a: => A): LExit[Any, Throwable, Any, A] =
    LExit.fromZIO(ZIO.attempt(a))
}
