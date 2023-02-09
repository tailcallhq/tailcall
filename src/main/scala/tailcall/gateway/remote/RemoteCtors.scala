package tailcall.gateway.remote

import zio.schema.Schema

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

  // TODO: Add a custom implementation for arity=2
  def fromFunction2[A0, A1, B](ab: (Remote[A0], Remote[A1]) => Remote[B]): Remote[(A0, A1) => B] =
    ???

  def seq[A](a: Seq[Remote[A]]): Remote[Seq[A]] =
    Remote.unsafe.attempt(DynamicEval.seq(a.map(_.compile)))

  def either[E, A](a: Either[Remote[E], Remote[A]]): Remote[Either[E, A]] =
    Remote
      .unsafe
      .attempt(DynamicEval.either(a match {
        case Left(value)  => Left(value.compile)
        case Right(value) => Right(value.compile)
      }))
}
