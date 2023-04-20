package tailcall.runtime.model

import caliban.parsing.adt.Definition.TypeSystemDefinition.DirectiveLocation
import tailcall.runtime.http.Method
import tailcall.runtime.lambda.~>>
import tailcall.runtime.model.Steps.Step
import tailcall.runtime.{DirectiveCodec, DirectiveDefinitionBuilder, JsonT}
import zio.json._
import zio.json.ast.Json
import zio.schema.{DeriveSchema, DynamicValue, Schema}

final case class Steps(value: List[Step]) {
  def ++(other: Steps): Steps = Steps(value ++ other.value)
  def :+(other: Step): Steps  = Steps(value :+ other)
  def +:(other: Step): Steps  = Steps(other +: value)
}

object Steps {
  sealed trait Step

  object Step {
    def objPath(spec: (String, List[String])*): Step = Transform(JsonT.objPath(spec: _*))

    def constant(a: Json): Step = Transform(JsonT.Constant(a))

    def transform(jsonT: JsonT): Step = Transform(jsonT)

    def function(f: DynamicValue ~>> DynamicValue): Step = LambdaFunction(f)

    @jsonHint("lambda")
    final case class LambdaFunction(f: DynamicValue ~>> DynamicValue) extends Step

    object LambdaFunction {
      implicit lazy val jsonCodec: JsonCodec[LambdaFunction] = zio.schema.codec.JsonCodec
        .jsonCodec(Schema[DynamicValue ~>> DynamicValue]).transform(LambdaFunction(_), _.f)
    }

    @jsonHint("http")
    final case class Http(
      path: Path,
      method: Option[Method] = None,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None,
      body: Option[String] = None,
    ) extends Step {
      def withInput(input: Option[TSchema]): Http = copy(input = input)

      def withMethod(method: Method): Http = copy(method = Option(method))

      def withOutput(output: Option[TSchema]): Http = copy(output = output)

      def withBody(body: Option[String]): Http = copy(body = body)
    }

    @jsonHint("transform")
    final case class Transform(transformation: JsonT) extends Step

    object Transform {
      implicit val jsonCodec: JsonCodec[Transform] = JsonCodec[JsonT].transform(Transform(_), _.transformation)
    }

    object Http {
      private val jsonCodec: JsonCodec[Http] = DeriveJsonCodec.gen[Http]

      def fromEndpoint(endpoint: Endpoint): Http =
        Http(
          path = endpoint.path,
          method = Option(endpoint.method),
          input = endpoint.input,
          output = endpoint.output,
          body = endpoint.body.flatMap(Mustache.syntax.printString(_).toOption),
        )

      implicit val directive: DirectiveCodec[Http] = DirectiveCodec.fromJsonCodec("http", jsonCodec)
    }

    implicit lazy val jsonCodec: JsonCodec[Step] = DeriveJsonCodec.gen[Step]
  }

  implicit val jsonCodec: JsonCodec[Steps] = DeriveJsonCodec.gen[Steps]

  implicit val schema: Schema[Steps]                         = DeriveSchema.gen[Steps]
  implicit val directiveCodec: DirectiveCodec[Steps]         = DirectiveCodec.fromJsonCodec("steps", jsonCodec)
  def directiveDefinition: DirectiveDefinitionBuilder[Steps] =
    DirectiveDefinitionBuilder.make[Steps].withLocations(DirectiveLocation.TypeSystemDirectiveLocation.FIELD_DEFINITION)
}
