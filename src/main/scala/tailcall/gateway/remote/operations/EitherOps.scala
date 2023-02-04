package tailcall.gateway.remote.operations

import tailcall.gateway.remote.Remote

trait EitherOps {
  implicit final class RemoteEitherOps[E, A](private val self: Remote[Either[E, A]]) {
    def diverge[B](f: Remote[E] => Remote[B], g: Remote[A] => Remote[B]): Remote[B] = ???
  }
}
