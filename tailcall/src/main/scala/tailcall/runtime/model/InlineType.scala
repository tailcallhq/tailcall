package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.json.JsonCodec
import zio.schema.annotation.caseName
import zio.schema.codec.JsonCodec.jsonCodec
import zio.schema.{DeriveSchema, Schema}

@caseName("inline")
final case class InlineType(path: List[String])

object InlineType {
  implicit val schema: Schema[InlineType]            = DeriveSchema.gen[InlineType]
  implicit val json: JsonCodec[InlineType]           = jsonCodec(schema)
  implicit val directive: DirectiveCodec[InlineType] = DirectiveCodec.gen[InlineType]
}
