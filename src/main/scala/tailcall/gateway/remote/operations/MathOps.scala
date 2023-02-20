package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.Lambda
import tailcall.gateway.remote.Remote
import tailcall.gateway.lambda.Numeric

trait MathOps {
  implicit final class RemoteMathOps[A](val self: Remote[A]) {
    def +(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.add(self.toLambda, other.toLambda))

    def -(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.subtract(self.toLambda, other.toLambda))

    def *(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.multiply(self.toLambda, other.toLambda))

    def /(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.divide(self.toLambda, other.toLambda))

    def %(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.modulo(self.toLambda, other.toLambda))

    def >(other: Remote[A])(implicit ev: Numeric[A]): Remote[Boolean] =
      Remote(Lambda.gt(self.toLambda, other.toLambda))

    def unary_-(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.negate(self.toLambda))
  }

}
