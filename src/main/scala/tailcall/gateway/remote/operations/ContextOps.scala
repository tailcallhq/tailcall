package tailcall.gateway.remote.operations

import tailcall.gateway.ast.Context
import tailcall.gateway.remote.{DynamicEval, Remote}
import zio.schema.DynamicValue

trait ContextOps {

  implicit final class RemoteContextOps(private val self: Remote[Context]) {
    def value: Remote[DynamicValue] = Remote.unsafe.attempt(DynamicEval.contextValue(self.compile))

    def args: Remote[Map[String, DynamicValue]] =
      Remote.unsafe.attempt(DynamicEval.contextArgs(self.compile))

    def parent: Remote[Option[Context]] =
      Remote.unsafe.attempt(DynamicEval.contextParent(self.compile))
  }
}
