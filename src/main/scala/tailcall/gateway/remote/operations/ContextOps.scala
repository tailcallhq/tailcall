package tailcall.gateway.remote.operations

import tailcall.gateway.ast.Context
import tailcall.gateway.remote.{DynamicEval, Remote}
import zio.schema.DynamicValue

trait ContextOps {
  implicit final class RemoteContextOps(private val self: Remote[Context]) {
    def value: Remote[DynamicValue] =
      Remote.unsafe.attempt(ctx => DynamicEval.contextValue(self.compile(ctx)))

    def arg(name: String): Remote[Option[DynamicValue]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.contextArgs(self.compile(ctx), name))

    def parent: Remote[Option[Context]] =
      Remote.unsafe.attempt(ctx => DynamicEval.contextParent(self.compile(ctx)))
  }
}
