package tailcall.runtime.transcoder

import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.transcoder.Transcoder.Output

trait OrcToBlueprint {
  implicit val configToBlueprint: Transcoder[Config, Blueprint] =
    Transcoder(config => Output.succeed(config.toBlueprint))
}
