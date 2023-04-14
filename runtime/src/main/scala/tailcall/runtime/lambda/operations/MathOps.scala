package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.{Lambda, Numeric}

trait MathOps {
  implicit final class LambdaMathOps[A, B](val self: Lambda[A, B]) {
    def +(other: Lambda[A, B])(implicit ev: Numeric[B]): Lambda[A, B] = Lambda.math.add(self, other)

    def -(other: Lambda[A, B])(implicit ev: Numeric[B]): Lambda[A, B] = Lambda.math.sub(self, other)

    def *(other: Lambda[A, B])(implicit ev: Numeric[B]): Lambda[A, B] = Lambda.math.mul(self, other)

    def /(other: Lambda[A, B])(implicit ev: Numeric[B]): Lambda[A, B] = Lambda.math.div(self, other)

    def %(other: Lambda[A, B])(implicit ev: Numeric[B]): Lambda[A, B] = Lambda.math.mod(self, other)

    def >(other: Lambda[A, B])(implicit ev: Numeric[B]): Lambda[A, Boolean] = Lambda.math.gt(self, other)

    def unary_-(implicit ev: Numeric[B]): Lambda[A, B] = Lambda.math.neg(self)
  }
}
