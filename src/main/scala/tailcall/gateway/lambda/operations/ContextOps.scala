package tailcall.gateway.lambda.operations

import tailcall.gateway.ast.Context
import tailcall.gateway.lambda.DynamicEval.ContextOperations
import tailcall.gateway.lambda.{Lambda, Remote}
import zio.schema.DynamicValue

trait ContextOps {
  implicit final class Extensions(private val self: Remote[Context]) {
    def value: Remote[DynamicValue] =
      Lambda
        .unsafe
        .attempt(ctx =>
          ContextOperations(self.compile(ctx), ContextOperations.GetValue)
        )

    def arg(name: String): Remote[Option[DynamicValue]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          ContextOperations(self.compile(ctx), ContextOperations.GetArg(name))
        )

    def parent: Remote[Option[Context]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          ContextOperations(self.compile(ctx), ContextOperations.GetParent)
        )
  }
}
