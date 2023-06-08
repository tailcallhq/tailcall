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
import java.net.{URI, URL}

@caseName("unsafe")
final case class UnsafeSteps(steps: List[Operation]) {
  def compress: UnsafeSteps = UnsafeSteps(steps.map(_.compress))
}

object UnsafeSteps {
  implicit val jsonCodec: JsonCodec[UnsafeSteps] = DeriveJsonCodec.gen[UnsafeSteps]

  implicit val directive: DirectiveCodec[UnsafeSteps] = DirectiveCodec.fromJsonCodec("unsafe", jsonCodec)

  def apply(steps: Operation*): UnsafeSteps = UnsafeSteps(steps.toList)

  sealed trait Operation {
    self =>
    def compress: Operation
  }

  object Operation {
    implicit lazy val jsonCodec: JsonCodec[Operation] = DeriveJsonCodec.gen[Operation]

    def constant(a: Json): Operation = Transform(JsonT.Constant(a))

    def function(f: DynamicValue ~>> DynamicValue): Operation = LambdaFunction(f)

    def objPath(spec: (String, List[String])*): Operation = Transform(JsonT.objPath(spec: _*))

    def transform(jsonT: JsonT): Operation = Transform(jsonT)

    @jsonHint("lambda")
    final case class LambdaFunction(f: DynamicValue ~>> DynamicValue) extends Operation {
      override def compress: Operation = this
    }

    @jsonHint("http")
    final case class Http(
      path: Path,
      method: Option[Method] = None,
      query: Option[Map[String, String]] = None,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None,
      body: Option[String] = None,
      groupBy: Option[List[String]] = None,
      batchKey: Option[String] = None,
      baseURL: Option[URL] = None,
    ) extends Operation {
      self =>
      override def compress: Http = {
        val method  = self.method.filterNot(_ == Method.GET)
        val query   = self.query.filter(_.nonEmpty)
        val groupBy = self.groupBy.filter(_.nonEmpty)
        self.copy(method = method, query = query, groupBy = groupBy)
      }

      def withBatchKey(batchKey: String): Http = copy(batchKey = Option(batchKey))

      def withBody(body: Option[String]): Http = copy(body = body)

      def withGroupBy(groupBy: String*): Http = copy(groupBy = Option(groupBy.toList))

      def withInput(input: Option[TSchema]): Http = copy(input = input)

      def withMethod(method: Method): Http = copy(method = Option(method))

      def withOutput(output: Option[TSchema]): Http = copy(output = output)

      def withQuery(query: (String, String)*): Http = copy(query = Option(query.toMap))
    }

    @jsonHint("transform")
    final case class Transform(transformation: JsonT) extends Operation {
      override def compress: Operation = this
    }

    object LambdaFunction {
      implicit lazy val jsonCodec: JsonCodec[LambdaFunction] = zio.schema.codec.JsonCodec
        .jsonCodec(Schema[DynamicValue ~>> DynamicValue]).transform(LambdaFunction(_), _.f)
    }

    object Transform {
      implicit val jsonCodec: JsonCodec[Transform] = JsonCodec[JsonT].transform(Transform(_), _.transformation)
    }

    object Http {
      implicit val urlCodec: JsonCodec[URL]          = JsonCodec[String].transformOrFail[URL](
        string =>
          try Right(URI.create(string).toURL)
          catch { case _: Throwable => Left(s"Malformed url: ${string}") },
        _.toString,
      )

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

      def fromPath(path: String): Http = Http(Path.unsafe.fromString(path))
    }
  }
}
