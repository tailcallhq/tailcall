package tailcall.gateway.ast

import tailcall.gateway.remote.Remote

sealed trait Endpoint[-A, +B] {
  def compile[A1 <: A, B1 >: B]: Remote[A1 => B1] = ???
}

object Endpoint {
  final case class Pure[A](value: A)                        extends Endpoint[Any, A]
  final case class FromRemote[A, B](remote: Remote[A => B]) extends Endpoint[A, B]
  final case class Http[A, B](method: Method, route: Route) extends Endpoint[A, B]

  def fromRemote[A, B](remote: Remote[A => B]): Endpoint[A, B] = FromRemote(remote)
}
