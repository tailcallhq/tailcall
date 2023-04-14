package tailcall.runtime.remote.operations

import tailcall.runtime.JsonT
import tailcall.runtime.remote._
import zio.schema.{DynamicValue, Schema}

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps[R](private val self: Remote[R, DynamicValue]) {
    def toTyped[A](implicit schema: Schema[A]): Remote[R, Option[A]] = self >>> Remote.dynamic.toTyped[A]

    def path(name: String*): Remote[R, Option[DynamicValue]] = self >>> Remote.dynamic.path(name: _*)

    def toTypedPath[A](name: String*)(implicit schema: Schema[A]): Remote[R, Option[A]] =
      self.path(name: _*).flatMap(_.toTyped[A])

    def transform(jsonT: JsonT): Remote[R, DynamicValue] = self >>> Remote.dynamic.jsonTransform(jsonT)
  }
}
