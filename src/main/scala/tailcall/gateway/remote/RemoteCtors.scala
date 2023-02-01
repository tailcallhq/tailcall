package tailcall.gateway.remote

trait RemoteCtors {
  import Remote._
  def apply[A](a: A): Remote[A] = Literal(a)

  def fromFunction[A, B](ab: Remote[A] => Remote[B]): Remote[A => B] = {
    val id = Binding.make[A]
    Remote.RemoteFunction(id, ab(id))
  }
}
