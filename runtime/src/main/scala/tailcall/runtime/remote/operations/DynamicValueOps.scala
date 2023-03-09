package tailcall.runtime.remote.operations

import tailcall.runtime.lambda.Lambda
import tailcall.runtime.remote._
import zio.schema.{DynamicValue, Schema}

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(private val self: Remote[DynamicValue]) {
    def toTyped[A](implicit schema: Schema[A]): Remote[Option[A]] = Remote(self.toLambda >>> Lambda.dynamic.toTyped[A])

    def path(name: String*): Remote[Option[DynamicValue]] = Remote(self.toLambda >>> Lambda.dynamic.path(name: _*))

    def toTypedPath[A](name: String*)(implicit schema: Schema[A]): Remote[Option[A]] =
      self.path(name: _*).flatMap(_.toTyped[A])
  }
}
