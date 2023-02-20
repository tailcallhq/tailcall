package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.Math
import tailcall.gateway.lambda._

trait MathOps {
  implicit final class MathOps[A, B](val self: A ~> B)(implicit
    tag: Numeric[B]
  ) {
    def >(other: A ~> B): A ~> Boolean =
      Lambda
        .unsafe
        .attempt(ctx =>
          Math(
            Math.Binary(
              self.compile(ctx),
              other.compile(ctx),
              Math.Binary.GreaterThan
            ),
            tag.any
          )
        )

    def increment(implicit ctor: Constructor[B]): A ~> B =
      self + Lambda(tag.one)

    def decrement(implicit ctor: Constructor[B]): A ~> B =
      self - Lambda(tag.one)

    def +(other: A ~> B): A ~> B =
      Lambda
        .unsafe
        .attempt(ctx =>
          Math(
            Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Add),
            tag.any
          )
        )

    def -(other: A ~> B): A ~> B = self + other.negate

    def *(other: A ~> B): A ~> B =
      Lambda
        .unsafe
        .attempt(ctx =>
          Math(
            Math.Binary(
              self.compile(ctx),
              other.compile(ctx),
              Math.Binary.Multiply
            ),
            tag.any
          )
        )

    def /(other: A ~> B): A ~> B =
      Lambda
        .unsafe
        .attempt(ctx =>
          Math(
            Math.Binary(
              self.compile(ctx),
              other.compile(ctx),
              Math.Binary.Divide
            ),
            tag.any
          )
        )

    def %(other: A ~> B): A ~> B =
      Lambda
        .unsafe
        .attempt(ctx =>
          Math(
            Math.Binary(
              self.compile(ctx),
              other.compile(ctx),
              Math.Binary.Modulo
            ),
            tag.any
          )
        )

    def negate: A ~> B =
      Lambda
        .unsafe
        .attempt(ctx =>
          Math(Math.Unary(self.compile(ctx), Math.Unary.Negate), tag.any)
        )
  }
}
