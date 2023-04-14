package tailcall.runtime.remote

import tailcall.runtime.model.Endpoint
import tailcall.runtime.remote.{Lambda, ~>}
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service.EvaluationRuntime
import zio.ZIO
import zio.schema.{DynamicValue, Schema}

/**
 * Remote[A] Allows for any arbitrary computation that can
 * be serialized and when evaluated produces a result of
 * type A. This is the lowest level primitive that’s
 * extremely powerful. We use this inside the compiler to
 * convert the composition logic into some form of a Remote.
 */
final case class Remote[+A](toLambda: Any ~> A) {
  self =>
  def debug(prefix: String): Remote[A] = Remote(toLambda >>> Lambda.unsafe.debug(prefix))

  def evaluate: ZIO[EvaluationRuntime with HttpDataLoader, Throwable, A] = toLambda.evaluate {}

  def toDynamic[A1 >: A](implicit ev: Schema[A1]): Remote[DynamicValue] =
    Remote(self.toLambda >>> Lambda.dynamic.toDynamic)
}

object Remote {
  def apply[A](a: => A)(implicit c: Schema[A]): Remote[A] = Remote(Lambda(a))

  def bind[A, B](ab: Remote[A] => Remote[B]): Remote[A] => Remote[B] = a => Remote(a.toLambda >>> Remote.toLambda(ab))

  def die(reason: String): Remote[Nothing] = Remote(Lambda.unsafe.die(reason))

  def fromLambda[A, B](ab: A ~> B): Remote[A] => Remote[B] = a => Remote(a.toLambda >>> ab)

  def fromOption[A](a: Option[Remote[A]]): Remote[Option[A]] = Remote(Lambda.option(a.map(_.toLambda)))

  def fromEndpoint(endpoint: Endpoint, input: Remote[DynamicValue]): Remote[DynamicValue] =
    Remote(input.toLambda >>> Lambda.unsafe.fromEndpoint(endpoint))

  def toLambda[A, B](ab: Remote[A] => Remote[B]): A ~> B = Lambda.fromLambdaFunction[A, B](a => ab(Remote(a)).toLambda)

  implicit def schema[A]: Schema[Remote[A]] = Schema[Any ~> A].transform(Remote(_), _.toLambda)

  implicit def schemaFunction[A, B]: Schema[Remote[A] => Remote[B]] =
    Schema[A ~> B].transform[Remote[A] => Remote[B]](Remote.fromLambda, Remote.toLambda(_))
}
