package tailcall.gateway.remote

import zio.schema.Schema

trait RemoteCtors {
  import Remote._
  def apply[A](a: A)(implicit schema: Schema[A]): Remote[A] = Literal(a, schema)

  def fromFunction[A, B](ab: Remote[A] => Remote[B]): Remote[A => B] = {
    val id = Binding.make[A]
    Remote.RemoteFunction(id, ab(id))
  }
}
