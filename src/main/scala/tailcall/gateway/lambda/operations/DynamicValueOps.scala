package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.DynamicValueOperations
import tailcall.gateway.lambda.{DynamicEval, Lambda, Remote}
import zio.Chunk
import zio.schema.DynamicValue

trait DynamicValueOps {
  implicit final class Extensions(private val self: Remote[DynamicValue]) {
    def path(fields: String*): Remote[Option[DynamicValue]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.Path(Chunk.from(fields))
          )
        )

    def asString: Remote[Option[String]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsString
          )
        )

    def asBoolean: Remote[Option[Boolean]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsBoolean
          )
        )

    def asInt: Remote[Option[Int]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsInt
          )
        )

    def asLong: Remote[Option[Long]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsLong
          )
        )

    def asDouble: Remote[Option[Double]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsDouble
          )
        )

    def asFloat: Remote[Option[Float]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsFloat
          )
        )

    def asList: Remote[Option[List[DynamicValue]]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsList
          )
        )

    def asMap: Remote[Option[Map[DynamicValue, DynamicValue]]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          DynamicEval.DynamicValueOperations(
            self.compile(ctx),
            DynamicValueOperations.AsMap
          )
        )
  }
}
