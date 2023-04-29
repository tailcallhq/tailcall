package tailcall.runtime.model

import tailcall.runtime.http.Method
import tailcall.runtime.lambda.~>>
import tailcall.runtime.model.Mustache.MustacheExpression
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.{DirectiveCodec, JsonT}
import zio.json._
import zio.json.ast.Json
import zio.schema.annotation.caseName
import zio.schema.{DynamicValue, Schema}

@caseName("unsafe")
final case class UnsafeSteps(steps: List[Operation]) {
  def compress: UnsafeSteps = UnsafeSteps(steps.map(_.compress))
}

object UnsafeSteps {
  implicit val jsonCodec: JsonCodec[UnsafeSteps] = DeriveJsonCodec.gen[UnsafeSteps]

  implicit val directiveCodec: DirectiveCodec[UnsafeSteps] = DirectiveCodec.fromJsonCodec("unsafe", jsonCodec)

  def apply(steps: Operation*): UnsafeSteps = UnsafeSteps(steps.toList)

  sealed trait Operation {
    self =>
    def compress: Operation =
      self match {
        case step: Operation.Http => step.compress
        case step                 => step
      }
  }

  object Operation {
    implicit lazy val jsonCodec: JsonCodec[Operation] = DeriveJsonCodec.gen[Operation]

    def constant(a: Json): Operation = Transform(JsonT.Constant(a))

    def function(f: DynamicValue ~>> DynamicValue): Operation = LambdaFunction(f)

    def objPath(spec: (String, List[String])*): Operation = Transform(JsonT.objPath(spec: _*))

    def transform(jsonT: JsonT): Operation = Transform(jsonT)

    @jsonHint("lambda")
    final case class LambdaFunction(f: DynamicValue ~>> DynamicValue) extends Operation

    @jsonHint("http")
    final case class Http(
      path: Path,
      method: Option[Method] = None,
      query: Option[Map[String, List[String]]] = None,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None,
      body: Option[String] = None,
    ) extends HttpOperation with Operation {
      self =>
      def withBody(body: Option[String]): Http = copy(body = body)

      def withInput(input: Option[TSchema]): Http = copy(input = input)

      def withMethod(method: Method): Http = copy(method = Option(method))

      def withOutput(output: Option[TSchema]): Http = copy(output = output)

      override def compress: Http = self.copy(method = self.compressMethod).copy(query = self.compressQuery)
    }

    @jsonHint("transform")
    final case class Transform(transformation: JsonT) extends Operation

    object LambdaFunction {
      implicit lazy val jsonCodec: JsonCodec[LambdaFunction] = zio.schema.codec.JsonCodec
        .jsonCodec(Schema[DynamicValue ~>> DynamicValue]).transform(LambdaFunction(_), _.f)
    }

    object Transform {
      implicit val jsonCodec: JsonCodec[Transform] = JsonCodec[JsonT].transform(Transform(_), _.transformation)
    }

    object Http {
      implicit val jsonCodec: JsonCodec[Http]      = DeriveJsonCodec.gen[Http]
      implicit val directive: DirectiveCodec[Http] = DirectiveCodec.fromJsonCodec("http", jsonCodec)

      def fromEndpoint(endpoint: Endpoint): Http =
        Http(
          path = endpoint.path,
          method = Option(endpoint.method),
          input = endpoint.input,
          output = endpoint.output,
          body = endpoint.body.flatMap(MustacheExpression.syntax.printString(_).toOption),
        )
    }
  }
}
