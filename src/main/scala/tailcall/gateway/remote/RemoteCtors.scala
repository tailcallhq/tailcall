package tailcall.gateway.remote

trait RemoteCtors {
  import Remote._
  def apply[A](a: A): Remote[A]                                     = Literal(a)
  def fromFunction[A, B](f: Remote[A] => Remote[B]): Remote[A => B] = Closure.make(f)
}
