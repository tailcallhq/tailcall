package tailcall.gateway.remote

import tailcall.gateway.lambda.{Lambda, ~>}
import tailcall.gateway.service.EvaluationRuntime
import zio.ZIO
import zio.schema.{DynamicValue, Schema}

/**
 * Remote[A] Allows for any arbitrary computation that can
 * be serialized and when evaluated produces a result of
 * type A. This is the lowest level primitive that’s
 * extremely powerful. We use this inside the compiler to
 * convert the composition logic into some form of a Remote.
 */
sealed trait Remote[+A] {
  self =>
  final def toLambda: Any ~> A                       = Remote.toLambda(self)
  def evaluate: ZIO[EvaluationRuntime, Throwable, A] = toLambda.evaluate {}

  def toDynamic[A1 >: A](implicit ev: Schema[A1]): Remote[DynamicValue] =
    Remote(self.toLambda >>> Lambda.dynamic.toDynamic)

  def debug(message: String): Remote[A] = Remote(self.toLambda >>> Lambda.debug(message))
}

object Remote {
  final case class FromLambda[A](lambda: Any ~> A) extends Remote[A]

  def apply[A](a: => A)(implicit c: Schema[A]): Remote[A] = FromLambda(Lambda(a))

  def apply[A](a: Any ~> A): Remote[A] = FromLambda(a)

  def bind[A, B](ab: Remote[A] => Remote[B]): Remote[A] => Remote[B] = a => Remote(a.toLambda >>> Remote.toLambda(ab))

  def die(reason: String): Remote[Nothing] = Remote(Lambda.die(reason))

  def fromLambda[A, B](ab: A ~> B): Remote[A] => Remote[B] = a => Remote(a.toLambda >>> ab)

  def fromOption[A](a: Option[Remote[A]]): Remote[Option[A]] = Remote(Lambda.option(a.map(_.toLambda)))

  def toLambda[A, B](ab: Remote[A] => Remote[B]): A ~> B = Lambda.fromLambdaFunction[A, B](a => ab(Remote(a)).toLambda)

  def toLambda[A](remote: Remote[A]): Any ~> A = remote match { case FromLambda(lambda) => lambda }

  implicit def schema[A]: Schema[Remote[A]] = Schema[Any ~> A].transform(Remote(_), _.toLambda)

  implicit def schemaFunction[A, B]: Schema[Remote[A] => Remote[B]] =
    Schema[A ~> B].transform[Remote[A] => Remote[B]](Remote.fromLambda, Remote.toLambda(_))
}
