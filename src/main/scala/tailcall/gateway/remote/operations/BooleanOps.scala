package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.Lambda
import tailcall.gateway.remote.Remote

trait BooleanOps {
  implicit final class RemoteBooleanOps(val self: Remote[Boolean]) {
    def &&(other: Remote[Boolean]): Remote[Boolean] =
      Remote(Lambda.and(self.toLambda, other.toLambda))

    def ||(other: Remote[Boolean]): Remote[Boolean] =
      Remote(Lambda.or(self.toLambda, other.toLambda))

    def unary_! : Remote[Boolean] = Remote(Lambda.not(self.toLambda))

    def diverge[A](isTrue: Remote[A], isFalse: Remote[A]): Remote[A] =
      Remote(Lambda.diverge(self.toLambda, isTrue.toLambda, isFalse.toLambda))
  }
}
