package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.Lambda
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(private val self: Remote[DynamicValue]) {
    def asSeq: Remote[Option[Seq[DynamicValue]]]               = Remote(self.toLambda >>> Lambda.dynamic.asSeq)
    def asMap: Remote[Option[Map[DynamicValue, DynamicValue]]] = Remote(self.toLambda >>> Lambda.dynamic.asMap)
    def asString: Remote[Option[String]]                       = Remote(self.toLambda >>> Lambda.dynamic.asString)
    def asInt: Remote[Option[Int]]                             = Remote(self.toLambda >>> Lambda.dynamic.asInt)
    def asBoolean: Remote[Option[Boolean]]                     = Remote(self.toLambda >>> Lambda.dynamic.asBoolean)
  }
}
