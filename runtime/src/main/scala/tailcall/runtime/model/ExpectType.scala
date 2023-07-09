package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.json.{DeriveJsonCodec, JsonCodec}
import zio.schema.annotation.caseName
import zio.schema.{DeriveSchema, Schema}

@caseName("expect")
final case class ExpectType(output: String)

object ExpectType {
  implicit val schema: Schema[ExpectType]            = DeriveSchema.gen[ExpectType]
  implicit val jsonCodec: JsonCodec[ExpectType]      = DeriveJsonCodec.gen[ExpectType]
  implicit val directive: DirectiveCodec[ExpectType] = DirectiveCodec.fromSchema(schema)
}
