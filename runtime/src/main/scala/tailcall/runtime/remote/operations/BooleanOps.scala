package tailcall.runtime.remote.operations

import tailcall.runtime.remote.Remote

trait BooleanOps {
  implicit final class RemoteBooleanOps[R](val self: Remote[R, Boolean]) {
    def &&(other: Remote[R, Boolean]): Remote[R, Boolean] = Remote.logic.and(self, other)

    def ||(other: Remote[R, Boolean]): Remote[R, Boolean] = Remote.logic.or(self, other)

    def unary_! : Remote[R, Boolean] = Remote.logic.not(self)

    def diverge[A](isTrue: Remote[R, A], isFalse: Remote[R, A]): Remote[R, A] = Remote.logic.cond(self)(isTrue, isFalse)
  }
}
