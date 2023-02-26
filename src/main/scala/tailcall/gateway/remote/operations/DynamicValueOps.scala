package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.{Constructor, Lambda}
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(private val self: Remote[DynamicValue]) {
    def toTyped[A](implicit ctor: Constructor[A]): Remote[Option[A]] =
      Remote(self.toLambda >>> Lambda.dynamic.toTyped[A])
  }
}
