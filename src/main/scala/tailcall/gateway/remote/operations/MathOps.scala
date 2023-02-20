package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.{Lambda, Numeric}
import tailcall.gateway.remote.Remote

trait MathOps {
  implicit final class RemoteMathOps[A](val self: Remote[A]) {
    def +(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] = Remote(Lambda.math.add(self.toLambda, other.toLambda))

    def -(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.math.subtract(self.toLambda, other.toLambda))

    def *(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.math.multiply(self.toLambda, other.toLambda))

    def /(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.math.divide(self.toLambda, other.toLambda))

    def %(other: Remote[A])(implicit ev: Numeric[A]): Remote[A] =
      Remote(Lambda.math.modulo(self.toLambda, other.toLambda))

    def >(other: Remote[A])(implicit ev: Numeric[A]): Remote[Boolean] =
      Remote(Lambda.math.gt(self.toLambda, other.toLambda))

    def unary_-(implicit ev: Numeric[A]): Remote[A] = Remote(Lambda.math.negate(self.toLambda))
  }

}
