package tailcall.runtime.remote.operations

import tailcall.runtime.remote.{Numeric, Remote}

trait MathOps {
  implicit final class RemoteMathOps[A, B](val self: Remote[A, B]) {
    def +(other: Remote[A, B])(implicit ev: Numeric[B]): Remote[A, B] = Remote.math.add(self, other)

    def -(other: Remote[A, B])(implicit ev: Numeric[B]): Remote[A, B] = Remote.math.sub(self, other)

    def *(other: Remote[A, B])(implicit ev: Numeric[B]): Remote[A, B] = Remote.math.mul(self, other)

    def /(other: Remote[A, B])(implicit ev: Numeric[B]): Remote[A, B] = Remote.math.div(self, other)

    def %(other: Remote[A, B])(implicit ev: Numeric[B]): Remote[A, B] = Remote.math.mod(self, other)

    def >(other: Remote[A, B])(implicit ev: Numeric[B]): Remote[A, Boolean] = Remote.math.gt(self, other)

    def unary_-(implicit ev: Numeric[B]): Remote[A, B] = Remote.math.neg(self)
  }
}
