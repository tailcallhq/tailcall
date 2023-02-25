package tailcall.gateway

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
      val document = Document(List(
        Document.SchemaDefinition(query = Some("Query")),
        Document.ObjectTypeDefinition(
          "Query",
          List(
            Document
              .FieldDefinition(name = "id", List(), Document.NamedType("Int", true), _ => Remote(DynamicValue(100)))
          )
        )
      ))

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

  def execute(doc: Document)(query: String): ZIO[DocumentGraphQLGenerator, Throwable, String] =
    for {
      graphQL     <- doc.toGraphQL
      interpreter <- graphQL.interpreter
      result <- interpreter.execute(query, skipValidation = true) // TODO: enable validation after __type is available
      _      <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
}
