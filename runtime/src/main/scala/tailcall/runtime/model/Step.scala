package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import tailcall.runtime.http.Method
import zio.json._
import zio.json.ast.Json

sealed trait Step

object Step {
  def const[A: JsonEncoder](a: A): Step = Constant(a.toJsonAST.toOption.get)

  @jsonHint("http")
  final case class Http(
    path: Path,
    method: Option[Method] = None,
    input: Option[TSchema] = None,
    output: Option[TSchema] = None,
  ) extends Step {
    def withInput(input: Option[TSchema]): Http   = copy(input = input)
    def withMethod(method: Method): Http          = copy(method = Option(method))
    def withOutput(output: Option[TSchema]): Http = copy(output = output)
  }

  @jsonHint("const")
  final case class Constant(json: Json) extends Step

  @jsonHint("objectPath")
  final case class ObjPath(map: Map[String, List[String]]) extends Step

  @jsonHint("toPair")
  case object ToPair extends Step {
    implicit lazy val jsonCodec: JsonCodec[ToPair.type] = DeriveJsonCodec.gen[ToPair.type]
    implicit val directive: DirectiveCodec[ToPair.type] = DirectiveCodec.fromJsonCodec("toPair", jsonCodec)
  }

  @jsonHint("identity")
  case object Identity extends Step {
    implicit lazy val jsonCodec: JsonCodec[Identity.type] = DeriveJsonCodec.gen[Identity.type]
    implicit val directive: DirectiveCodec[Identity.type] = DirectiveCodec.fromJsonCodec("identity", jsonCodec)
  }

  object Http {
    private val jsonCodec: JsonCodec[Http] = DeriveJsonCodec.gen[Http]

    def fromEndpoint(endpoint: Endpoint): Http   =
      Http(path = endpoint.path, method = Option(endpoint.method), input = endpoint.input, output = endpoint.output)
    implicit val directive: DirectiveCodec[Http] = DirectiveCodec.fromJsonCodec("http", jsonCodec)
  }

  object Constant {
    implicit val codec: JsonCodec[Constant] = JsonCodec(Json.encoder, Json.decoder).transform(Constant(_), _.json)
    implicit val directive: DirectiveCodec[Constant] = DirectiveCodec.fromJsonCodec("const", codec)
  }

  implicit lazy val jsonCodec: JsonCodec[Step] = DeriveJsonCodec.gen[Step]

  // TODO: this should be auto-generated
  implicit lazy val directive: DirectiveCodec[List[Step]] = DirectiveCodec.fromJsonListCodec("steps", jsonCodec)

  object ObjPath {
    def apply(map: (String, List[String])*): ObjPath = ObjPath(map.toMap)
    implicit val codec: JsonCodec[ObjPath]           = JsonCodec[Map[String, List[String]]].transform(ObjPath(_), _.map)
    implicit val directive: DirectiveCodec[ObjPath]  = DirectiveCodec.fromJsonCodec("objectPath", codec)
  }
}
