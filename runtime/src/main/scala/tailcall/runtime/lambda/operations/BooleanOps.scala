package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.Lambda

trait BooleanOps {
  implicit final class LambdaBooleanOps[A](val self: Lambda[A, Boolean]) {
    def &&(other: Lambda[A, Boolean]): Lambda[A, Boolean] = Lambda.logic.and(self, other)

    def ||(other: Lambda[A, Boolean]): Lambda[A, Boolean] = Lambda.logic.or(self, other)

    def unary_! : Lambda[A, Boolean] = Lambda.logic.not(self)

    def diverge[B](isTrue: Lambda[A, B], isFalse: Lambda[A, B]): Lambda[A, B] = Lambda.logic.cond(self)(isTrue, isFalse)
  }
}
