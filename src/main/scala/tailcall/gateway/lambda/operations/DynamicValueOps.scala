package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.DynamicValueOperations
import tailcall.gateway.lambda.{DynamicEval, Remote}
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
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.Path(Chunk.from(fields))
          )
        )

    def asString: Remote[Option[String]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsString
          )
        )

    def asBoolean: Remote[Option[Boolean]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsBoolean
          )
        )

    def asInt: Remote[Option[Int]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsInt
          )
        )

    def asLong: Remote[Option[Long]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsLong
          )
        )

    def asDouble: Remote[Option[Double]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsDouble
          )
        )

    def asFloat: Remote[Option[Float]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsFloat
          )
        )

    def asList: Remote[Option[List[DynamicValue]]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsList
          )
        )

    def asMap: Remote[Option[Map[DynamicValue, DynamicValue]]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsMap
          )
        )
  }
}
