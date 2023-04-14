package tailcall.runtime.remote.operations

import tailcall.runtime.remote.{Numeric, Remote}

trait MathOps {
  implicit final class RemoteMathOps[R, A](val self: Remote[R, A]) {
    def +(other: Remote[R, A])(implicit ev: Numeric[A]): Remote[R, A] = Remote.math.add(self, other)

    def -(other: Remote[R, A])(implicit ev: Numeric[A]): Remote[R, A] = Remote.math.sub(self, other)

    def *(other: Remote[R, A])(implicit ev: Numeric[A]): Remote[R, A] = Remote.math.mul(self, other)

    def /(other: Remote[R, A])(implicit ev: Numeric[A]): Remote[R, A] = Remote.math.div(self, other)

    def %(other: Remote[R, A])(implicit ev: Numeric[A]): Remote[R, A] = Remote.math.mod(self, other)

    def >(other: Remote[R, A])(implicit ev: Numeric[A]): Remote[R, Boolean] = Remote.math.gt(self, other)

    def unary_-(implicit ev: Numeric[A]): Remote[R, A] = Remote.math.neg(self)
  }
}
