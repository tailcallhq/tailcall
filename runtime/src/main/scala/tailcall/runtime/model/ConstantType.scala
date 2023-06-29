package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.json.JsonCodec
import zio.schema.annotation.caseName
import zio.schema.codec.JsonCodec.jsonCodec
import zio.schema.{DeriveSchema, Schema}

@caseName("constant")
final case class ConstantType(value: String)

object ConstantType {
  implicit val schema: Schema[ConstantType]            = DeriveSchema.gen[ConstantType]
  implicit val json: JsonCodec[ConstantType]           = jsonCodec(schema)
  implicit val directive: DirectiveCodec[ConstantType] = DirectiveCodec.gen[ConstantType]
}
