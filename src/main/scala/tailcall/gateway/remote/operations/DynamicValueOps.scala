package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}
import zio.Chunk
import zio.schema.DynamicValue

trait DynamicValueOps {
  implicit final class RemoteDynamicValueOps(
    private val self: Remote[DynamicValue]
  ) {
    def path(fields: String*): Remote[Option[DynamicValue]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.dynamicValuePath(self.compile(ctx), Chunk.from(fields))
        )

    def asString: Remote[Option[String]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsString(self.compile(ctx)))

    def asBoolean: Remote[Option[Boolean]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsBoolean(self.compile(ctx)))

    def asInt: Remote[Option[Int]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsInt(self.compile(ctx)))

    def asLong: Remote[Option[Long]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsLong(self.compile(ctx)))

    def asDouble: Remote[Option[Double]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsDouble(self.compile(ctx)))

    def asFloat: Remote[Option[Float]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsFloat(self.compile(ctx)))

    def asList: Remote[Option[List[DynamicValue]]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsList(self.compile(ctx)))

    def asMap: Remote[Option[Map[DynamicValue, DynamicValue]]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.dynamicValueAsMap(self.compile(ctx)))
  }
}
