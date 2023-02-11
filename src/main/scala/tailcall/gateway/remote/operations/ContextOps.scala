package tailcall.gateway.remote.operations

import tailcall.gateway.ast.Context
import tailcall.gateway.remote.{DynamicEval, Remote}
import zio.schema.DynamicValue

trait ContextOps {
  implicit final class RemoteContextOps(private val self: Remote[Context]) {
    def value: Remote[DynamicValue] = Remote.unsafe.attempt(DynamicEval.contextValue(self.compile))

    def arg(name: String): Remote[Option[DynamicValue]] =
      Remote.unsafe.attempt(DynamicEval.contextArgs(self.compile, name))

    def parent: Remote[Option[Context]] =
      Remote.unsafe.attempt(DynamicEval.contextParent(self.compile))
  }
}
