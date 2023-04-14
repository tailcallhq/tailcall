package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.{Lambda, Numeric, ~>}

trait MathOps {
  implicit final class LambdaMathOps[A, B](val self: A ~> B) {
    def +(other: A ~> B)(implicit ev: Numeric[B]): A ~> B = Lambda.math.add(self, other)

    def -(other: A ~> B)(implicit ev: Numeric[B]): A ~> B = Lambda.math.sub(self, other)

    def *(other: A ~> B)(implicit ev: Numeric[B]): A ~> B = Lambda.math.mul(self, other)

    def /(other: A ~> B)(implicit ev: Numeric[B]): A ~> B = Lambda.math.div(self, other)

    def %(other: A ~> B)(implicit ev: Numeric[B]): A ~> B = Lambda.math.mod(self, other)

    def >(other: A ~> B)(implicit ev: Numeric[B]): A ~> Boolean = Lambda.math.gt(self, other)

    def unary_-(implicit ev: Numeric[B]): A ~> B = Lambda.math.neg(self)
  }
}
