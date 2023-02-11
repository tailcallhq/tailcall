package tailcall.gateway.remote

import tailcall.gateway.ast.Endpoint
import zio.schema.{DynamicValue, Schema}

trait RemoteCtors {
  def apply[A](a: A)(implicit schema: Schema[A]): Remote[A] =
    Remote.unsafe.attempt(DynamicEval.Literal(schema.toDynamic(a), schema.ast))

  def fromFunction[A, B](ab: Remote[A] => Remote[B]): Remote[A => B] =
    Remote
      .unsafe
      .attempt {
        val id = DynamicEval.binding
        DynamicEval.bind(id, ab(Remote.unsafe.attempt[A](id)).compile)
      }

  def fromSeq[A](a: Seq[Remote[A]]): Remote[Seq[A]] =
    Remote.unsafe.attempt(DynamicEval.seq(a.map(_.compile)))

  def fromEither[E, A](a: Either[Remote[E], Remote[A]]): Remote[Either[E, A]] =
    Remote
      .unsafe
      .attempt(DynamicEval.either(a match {
        case Left(value)  => Left(value.compile)
        case Right(value) => Right(value.compile)
      }))

  def fromOption[A](a: Option[Remote[A]]): Remote[Option[A]] =
    Remote.unsafe.attempt(DynamicEval.option(a.map(_.compile)))

  def fromEndpoint(endpoint: Endpoint): Remote[DynamicValue => DynamicValue] =
    Remote.fromFunction[DynamicValue, DynamicValue](input =>
      Remote.unsafe.attempt(DynamicEval.endpoint(endpoint, input.compile))
    )

  def dynamicValue[A](a: A)(implicit schema: Schema[A]): Remote[DynamicValue] =
    Remote(Schema.toDynamic(a))

  def record(fields: (String, Remote[DynamicValue])*): Remote[DynamicValue] =
    Remote.unsafe.attempt(DynamicEval.record(fields.map { case (k, v) => k -> v.compile }))

  def die(msg: Remote[String]): Remote[Nothing] =
    Remote.unsafe.attempt(DynamicEval.die(msg.compile))

  def die(msg: String): Remote[Nothing] = die(Remote(msg))

  def batch(remote: Remote[DynamicValue], groupByKey: List[String]): Remote[DynamicValue] =
    Remote.unsafe.attempt(DynamicEval.batch(remote.compile, groupByKey))
}
