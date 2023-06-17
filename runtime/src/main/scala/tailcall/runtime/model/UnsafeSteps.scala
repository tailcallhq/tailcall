package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.json._
import zio.schema.annotation.caseName

@caseName("unsafe")
final case class UnsafeSteps(steps: List[Operation]) {
  def compress: UnsafeSteps = UnsafeSteps(steps.map(_.compress))
}

object UnsafeSteps {
  implicit val jsonCodec: JsonCodec[UnsafeSteps]      = DeriveJsonCodec.gen[UnsafeSteps]
  implicit val directive: DirectiveCodec[UnsafeSteps] = DirectiveCodec.fromJsonCodec("unsafe", jsonCodec)

  def apply(steps: Operation*): UnsafeSteps = UnsafeSteps(steps.toList)
}
