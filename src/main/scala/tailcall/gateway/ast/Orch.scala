package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema.Schema

/**
 * The core domain to perform API orchestration. It takes in
 * an input of type A and performs a series of steps to
 * produce an output of type B.
 */

sealed trait Orch[-A, +B] {
  final def compile[A1 <: A, B1 >: B]: Remote[A1 => B1] = ???
}

object Orch {
  final case class Pure[A](value: A)                        extends Orch[Any, A]
  final case class FromRemote[A, B](remote: Remote[A => B]) extends Orch[A, B]

  def fromRemote[A, B](remote: Remote[A => B]): Orch[A, B] = FromRemote(remote)

  implicit def schema[A, B]: Schema[Orch[A, B]] = Schema[Remote[A => B]]
    .transform(fromRemote(_), _.compile)
}
