package tailcall.runtime.remote.operations

import tailcall.runtime.JsonT
import tailcall.runtime.remote._
import zio.schema.{DynamicValue, Schema}

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps[A](private val self: Remote[A, DynamicValue]) {
    def toTyped[B](implicit schema: Schema[B]): Remote[A, Option[B]] = self >>> Remote.dynamic.toTyped[B]

    def path(name: String*): Remote[A, Option[DynamicValue]] = self >>> Remote.dynamic.path(name: _*)

    def toTypedPath[B](name: String*)(implicit schema: Schema[B]): Remote[A, Option[B]] =
      self.path(name: _*).flatMap(_.toTyped[B])

    def transform(jsonT: JsonT): Remote[A, DynamicValue] = self >>> Remote.dynamic.jsonTransform(jsonT)
  }
}
