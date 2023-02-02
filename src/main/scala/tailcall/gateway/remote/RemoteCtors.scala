package tailcall.gateway.remote

import zio.schema.Schema

trait RemoteCtors {
  def apply[A](a: A)(implicit schema: Schema[A]): Remote[A] = Remote.unsafe
    .attempt(DynamicEval.Literal(schema.toDynamic(a), schema.ast))

  def fromFunction[A, B](ab: Remote[A] => Remote[B]): Remote[A => B] = Remote.unsafe.attempt {
    val id = DynamicEval.Binding.make
    DynamicEval.EvalFunction(id, ab(Remote.unsafe.attempt[A](id)).compile)
  }
}
