package tailcall.gateway.remote.operations

import tailcall.gateway.ast.Context
import tailcall.gateway.remote.DynamicEval.ContextOperations
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

trait ContextOps {
  implicit final class RemoteContextOps(private val self: Remote[Context]) {
    def value: Remote[DynamicValue] =
      Remote
        .unsafe
        .attempt(ctx =>
          ContextOperations(self.compile(ctx), ContextOperations.GetValue)
        )

    def arg(name: String): Remote[Option[DynamicValue]] =
      Remote
        .unsafe
        .attempt(ctx =>
          ContextOperations(self.compile(ctx), ContextOperations.GetArg(name))
        )

    def parent: Remote[Option[Context]] =
      Remote
        .unsafe
        .attempt(ctx =>
          ContextOperations(self.compile(ctx), ContextOperations.GetParent)
        )
  }
}
