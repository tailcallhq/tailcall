package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait BooleanOps {
  implicit final class RemoteBooleanOps(val self: Remote[Boolean]) {
    def &&(other: Remote[Boolean]): Remote[Boolean] = Remote.unsafe
      .attempt(DynamicEval.and(self.compile, other.compile))

    def ||(other: Remote[Boolean]): Remote[Boolean] = Remote.unsafe
      .attempt(DynamicEval.or(self.compile, other.compile))

    def unary_! : Remote[Boolean] = Remote.unsafe.attempt(DynamicEval.not(self.compile))

    def diverge[A](isTrue: Remote[A], isFalse: Remote[A]): Remote[A] = Remote.unsafe
      .attempt(DynamicEval.diverge(self.compile, isTrue.compile, isFalse.compile))
  }
}
