package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.json.JsonCodec
import zio.schema.annotation.{caseName, fieldName}
import zio.schema.codec.JsonCodec.jsonCodec
import zio.schema.{DeriveSchema, Schema}

@caseName("extends")
final case class ExtendsType(@fieldName("type") typeOf: String)

object ExtendsType {
  implicit val schema: Schema[ExtendsType]            = DeriveSchema.gen[ExtendsType]
  implicit val json: JsonCodec[ExtendsType]           = jsonCodec(schema)
  implicit val directive: DirectiveCodec[ExtendsType] = DirectiveCodec.gen[ExtendsType]
}
