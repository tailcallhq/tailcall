package tailcall.gateway.ast

import zio.schema.{DeriveSchema, DynamicValue, Schema}

/**
 * A special purpose context that is used to orchestrate
 * APIs.
 */
final case class Context(
  value: DynamicValue,
  args: Map[String, DynamicValue] = Map.empty,
  parent: Option[Context] = None
)

object Context:
  implicit val schema: Schema[Context] = DeriveSchema.gen[Context]
