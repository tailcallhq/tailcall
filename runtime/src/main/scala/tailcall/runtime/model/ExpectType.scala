package tailcall.runtime.model

import tailcall.runtime.{DirectiveCodec, JsonT}
import zio.json.{DeriveJsonCodec, JsonCodec}
import zio.schema.annotation.caseName

@caseName("expect")
final case class ExpectType(output: JsonT.Constant)

object ExpectType {

  implicit val jsonCodec: JsonCodec[ExpectType]      = DeriveJsonCodec.gen[ExpectType]
  implicit val directive: DirectiveCodec[ExpectType] = DirectiveCodec.fromJsonCodec("expect", jsonCodec)
}
