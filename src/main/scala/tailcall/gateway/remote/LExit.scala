package tailcall.gateway.remote

import zio._

final case class LExit[-R, +E, -A, +B](exit: A => ZIO[R, E, B])
    extends (A => ZIO[R, E, B]) {
  self =>

  def apply(a: A): ZIO[R, E, B] = exit(a)

  def pipe[R1 <: R, E1 >: E, A1 >: B, B1](
    other: LExit[R1, E1, A1, B1]
  ): LExit[R1, E1, A, B1] = LExit(a => exit(a).flatMap(b => other.exit(b)))

  def >>>[R1 <: R, E1 >: E, A1 >: B, B1](
    other: LExit[R1, E1, A1, B1]
  ): LExit[R1, E1, A, B1] = self.pipe(other)

  def flatMap[R1 <: R, E1 >: E, A1 <: A, B1](
    f: B => LExit[R1, E1, A1, B1]
  ): LExit[R1, E1, A1, B1] = LExit(a => exit(a).flatMap(b => f(b).exit(a)))

  def map[C](f: B => C): LExit[R, E, A, C] = LExit(a => exit(a).map(f))
}

object LExit {
  def succeed[A](a: A): LExit[Any, Nothing, Any, A] = LExit(_ => ZIO.succeed(a))

  def fail[E](e: E): LExit[Any, E, Any, Nothing] = LExit(_ => ZIO.fail(e))

  def fromEither[E, A](either: Either[E, A]): LExit[Any, E, Any, A] =
    either.fold(fail, succeed)

  def fromZIO[R, E, B](effect: ZIO[R, E, B]): LExit[R, E, Any, B] =
    LExit(_ => effect)

  def input[A]: LExit[Any, Nothing, A, A] = LExit(ZIO.succeed(_))
}
