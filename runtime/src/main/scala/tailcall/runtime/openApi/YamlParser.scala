package tailcall.runtime.openApi

import tailcall.runtime.openApi.OpenapiModels.OpenapiDocument

object YamlParser {
  import io.circe._

  def parseFile(yamlString: String): Either[Error, OpenapiDocument] = {
    parser.parse(yamlString).flatMap(_.as[OpenapiDocument])
  }
}
