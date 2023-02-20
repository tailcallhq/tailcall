package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.Logical
import tailcall.gateway.lambda.{Lambda, ~>}

trait BooleanOps {
  implicit final class Extensions[A](val self: A ~> Boolean) {
    def &&(other: A ~> Boolean): A ~> Boolean =
      Lambda
        .unsafe
        .attempt(ctx =>
          Logical(
            Logical
              .Binary(self.compile(ctx), other.compile(ctx), Logical.Binary.And)
          )
        )

    def ||(other: A ~> Boolean): A ~> Boolean =
      Lambda
        .unsafe
        .attempt(ctx =>
          Logical(
            Logical
              .Binary(self.compile(ctx), other.compile(ctx), Logical.Binary.Or)
          )
        )

    def unary_! : A ~> Boolean =
      Lambda
        .unsafe
        .attempt(ctx =>
          Logical(Logical.Unary(self.compile(ctx), Logical.Unary.Not))
        )

    def diverge[B](isTrue: A ~> B, isFalse: A ~> B): A ~> B =
      Lambda
        .unsafe
        .attempt(ctx =>
          Logical(Logical.Unary(
            self.compile(ctx),
            Logical.Unary.Diverge(isTrue.compile(ctx), isFalse.compile(ctx))
          ))
        )
  }
}
