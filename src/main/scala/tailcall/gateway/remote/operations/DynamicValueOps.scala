package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.Lambda
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(private val self: Remote[DynamicValue]) {
    def asSeq: Remote[Option[Seq[DynamicValue]]] = Remote(self.toLambda >>> Lambda.dynamic.toTyped[Seq[DynamicValue]])
    def asMap: Remote[Option[Map[DynamicValue, DynamicValue]]] =
      Remote(self.toLambda >>> Lambda.dynamic.toTyped[Map[DynamicValue, DynamicValue]])
    def asString: Remote[Option[String]]   = Remote(self.toLambda >>> Lambda.dynamic.toTyped[String])
    def asInt: Remote[Option[Int]]         = Remote(self.toLambda >>> Lambda.dynamic.toTyped[Int])
    def asBoolean: Remote[Option[Boolean]] = Remote(self.toLambda >>> Lambda.dynamic.toTyped[Boolean])
  }
}
