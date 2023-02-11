package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}
import zio.Chunk
import zio.schema.DynamicValue

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(private val self: Remote[DynamicValue]) {
    def path(fields: String*): Remote[Option[DynamicValue]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValuePath(self.compile, Chunk.from(fields)))

    def asString: Remote[Option[String]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsString(self.compile))

    def asBoolean: Remote[Option[Boolean]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsBoolean(self.compile))

    def asInt: Remote[Option[Int]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsInt(self.compile))

    def asLong: Remote[Option[Long]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsLong(self.compile))

    def asDouble: Remote[Option[Double]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsDouble(self.compile))

    def asFloat: Remote[Option[Float]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsFloat(self.compile))

    def asList: Remote[Option[List[DynamicValue]]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsList(self.compile))

    def asMap: Remote[Option[Map[DynamicValue, DynamicValue]]] =
      Remote.unsafe.attempt(DynamicEval.dynamicValueAsMap(self.compile))
  }
}
