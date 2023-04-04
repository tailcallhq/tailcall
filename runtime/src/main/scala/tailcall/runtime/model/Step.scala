package tailcall.runtime.model

import tailcall.runtime.http.Method
import zio.json._
import zio.json.ast.Json

sealed trait Step
object Step {
  @jsonHint("http")
  final case class Http(
    path: Path,
    method: Option[Method] = None,
    input: Option[TSchema] = None,
    output: Option[TSchema] = None,
  ) extends Step {
    def withInput(input: Option[TSchema]): Http = copy(input = input)

    def withMethod(method: Method): Http = copy(method = Option(method))

    def withOutput(output: Option[TSchema]): Http = copy(output = output)
  }

  @jsonHint("const")
  final case class Constant(json: Json) extends Step

  @jsonHint("objectPath")
  final case class ObjPath(map: Map[String, List[String]]) extends Step

  object Http {
    def fromEndpoint(endpoint: Endpoint): Http =
      Http(path = endpoint.path, method = Option(endpoint.method), input = endpoint.input, output = endpoint.output)
  }

  object Constant {
    implicit val codec: JsonCodec[Constant] = JsonCodec(Json.encoder, Json.decoder).transform(Constant(_), _.json)
  }

  object ObjPath {
    def apply(map: (String, List[String])*): ObjPath = ObjPath(map.toMap)

    implicit val codec: JsonCodec[ObjPath] = JsonCodec[Map[String, List[String]]].transform(ObjPath(_), _.map)
  }

  implicit lazy val stepCodec: JsonCodec[Step] = DeriveJsonCodec.gen[Step]
}
