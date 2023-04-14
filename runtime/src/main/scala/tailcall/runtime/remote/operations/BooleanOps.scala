package tailcall.runtime.remote.operations

import tailcall.runtime.remote.Remote

trait BooleanOps {
  implicit final class RemoteBooleanOps[A](val self: Remote[A, Boolean]) {
    def &&(other: Remote[A, Boolean]): Remote[A, Boolean] = Remote.logic.and(self, other)

    def ||(other: Remote[A, Boolean]): Remote[A, Boolean] = Remote.logic.or(self, other)

    def unary_! : Remote[A, Boolean] = Remote.logic.not(self)

    def diverge[B](isTrue: Remote[A, B], isFalse: Remote[A, B]): Remote[A, B] = Remote.logic.cond(self)(isTrue, isFalse)
  }
}
