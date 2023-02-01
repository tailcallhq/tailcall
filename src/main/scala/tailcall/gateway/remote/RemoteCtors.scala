package tailcall.gateway.remote

trait RemoteCtors {
  import Remote._
  def apply[A](a: A): Remote[A] = Literal(a)
}
