package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.Lambda
import tailcall.gateway.remote.Remote
import zio.schema.{DynamicValue, Schema}

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(private val self: Remote[DynamicValue]) {
    def toTyped[A](implicit schema: Schema[A]): Remote[Option[A]] = Remote(self.toLambda >>> Lambda.dynamic.toTyped[A])

    def path(name: String*): Remote[Option[DynamicValue]] = Remote(self.toLambda >>> Lambda.dynamic.path(name: _*))
  }
}
