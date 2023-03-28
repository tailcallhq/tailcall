package tailcall.runtime.openApi

import io.circe.yaml.parser
import tailcall.runtime.openApi.OpenapiModels.OpenapiDocument

object YamlParser {
  import cats.implicits._
  import io.circe.Error

  def parseFile(yamlString: String): Either[Error, OpenapiDocument] = {
    parser.parse(yamlString).leftMap(err => err: Error).flatMap(_.as[OpenapiDocument])
  }
}
