package tailcall.runtime

import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.scala.Orc

package object transcoder extends TranscoderSyntax {
  implicit val orc2Blueprint: Transcoder[Orc, Blueprint] = Transcoder.make[Orc, Blueprint](Orc2Blueprint.toBlueprint)
  implicit val config2Blueprint: Transcoder[Config, Blueprint] = Transcoder.total(Config2Blueprint.toBlueprint)
}
