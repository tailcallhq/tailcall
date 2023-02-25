package tailcall.gateway

import caliban.CalibanError
import tailcall.gateway.ast.Document
import tailcall.gateway.remote.Remote
import tailcall.gateway.service._
import zio.ZIO
import zio.schema.DynamicValue
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

object DocumentStepGeneratorSpec extends ZIOSpecDefault {

  def spec = {
    suite("DocumentStepGenerator")(test("test") {
      val document = Document(List(Document.Definition.ObjectTypeDefinition(
        "Query",
        List(Document.Definition.FieldDefinition(
          name = "id",
          List(),
          Document.Type.NamedType("Int", true),
          Document.FieldResolver.FromContext(_ => Remote(DynamicValue(100)))
        ))
      )))

      val program = execute(document)("query {id}")

      assertZIO(program)(equalTo("""{"id":100}"""))
    }).provide(
      DocumentGraphQLGenerator.live,
      DocumentTypeGenerator.live,
      DocumentStepGenerator.live,
      EvaluationRuntime.live,
      EvaluationContext.live
    )
  }

  def execute(doc: Document)(query: String): ZIO[DocumentGraphQLGenerator, CalibanError.ValidationError, String] =
    for {
      graphQL     <- doc.toGraphQL
      interpreter <- graphQL.interpreter
      result      <- interpreter.execute(query)
    } yield result.data.toString
}
