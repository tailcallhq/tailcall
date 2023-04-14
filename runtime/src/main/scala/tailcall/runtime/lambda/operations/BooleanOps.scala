package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.{Lambda, ~>}

trait BooleanOps {
  implicit final class LambdaBooleanOps[A](val self: A ~> Boolean) {
    def &&(other: A ~> Boolean): A ~> Boolean = Lambda.logic.and(self, other)

    def ||(other: A ~> Boolean): A ~> Boolean = Lambda.logic.or(self, other)

    def unary_! : A ~> Boolean = Lambda.logic.not(self)

    def diverge[B](isTrue: A ~> B, isFalse: A ~> B): A ~> B = Lambda.logic.cond(self)(isTrue, isFalse)
  }
}
