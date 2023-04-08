package tailcall.runtime.model

import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec
import tailcall.runtime.http.Method
import tailcall.runtime.internal.TValid
import zio.json._
import zio.json.ast.Json

sealed trait Step {
  self =>
  final def toDirective: TValid[String, Directive] =
    self match {
      case self @ Step.Http(_, _, _, _) => Step.Http.directive.encode(self)
      case self @ Step.Constant(_)      => Step.Constant.directive.encode(self)
      case self @ Step.ObjPath(_)       => Step.ObjPath.directive.encode(self)
    }
}

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
