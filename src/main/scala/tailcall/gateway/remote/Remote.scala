package tailcall.gateway.remote

import tailcall.gateway.lambda.{Constructor, Lambda, LambdaRuntime, ~>}
import zio.ZIO
import zio.schema.Schema

/**
 * Remote[A] Allows for any arbitrary computation that can
 * be serialized and when evaluated produces a result of
 * type A. This is the lowest level primitive that’s
 * extremely powerful. We use this inside the compiler to
 * convert the composition logic into some form of a Remote.
 */
sealed trait Remote[+A] {
  self =>
  final def toLambda: Any ~> A                   = Remote.toLambda(self)
  def evaluate: ZIO[LambdaRuntime, Throwable, A] = toLambda.evaluate {}
}

object Remote {
  final case class FromLambda[A](lambda: Any ~> A) extends Remote[A]

  def apply[A](a: A)(implicit c: Constructor[A]): Remote[A] = FromLambda(Lambda(a))

  def apply[A](a: Any ~> A): Remote[A] = FromLambda(a)

  def toLambda[A](remote: Remote[A]): Any ~> A = remote match { case FromLambda(lambda) => lambda }

  implicit def schema[A]: Schema[Remote[A]] = Schema[Any ~> A].transform(Remote(_), _.toLambda)
}
