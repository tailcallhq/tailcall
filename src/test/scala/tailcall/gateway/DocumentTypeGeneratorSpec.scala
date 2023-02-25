package tailcall.gateway

import tailcall.gateway.ast.Document
import tailcall.gateway.remote.Remote
import tailcall.gateway.service._
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test._

object DocumentTypeGeneratorSpec extends ZIOSpecDefault {
  override def spec =
    suite("DocumentTypeGenerator")(
      test("document type generation") {
        val document = Document(List(
          Document.ObjectTypeDefinition(
            "Query",
            List(
              Document
                .FieldDefinition("test", List(), Document.NamedType("String", false), _ => Remote(DynamicValue("test")))
            )
          ),
          Document.SchemaDefinition(Some("Query"), None, None)
        ))
        val out      = document.toGraphQL.map(_.render)
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test: String
                          |}""".stripMargin
        assertZIO(out)(equalTo(expected))
      },
      test("document with InputValue") {
        val document = Document(List(
          Document.ObjectTypeDefinition(
            "Query",
            List(Document.FieldDefinition(
              "test",
              List(
                Document.InputValueDefinition("arg", Document.NamedType("String", false), Some(DynamicValue("test")))
              ),
              Document.NamedType("String", false),
              _ => Remote(DynamicValue("test"))
            ))
          ),
          Document.SchemaDefinition(Some("Query"), None, None)
        ))
        val out      = document.toGraphQL.map(_.render)
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertZIO(out)(equalTo(expected))
      }
    ).provide(
      DocumentGraphQLGenerator.live,
      DocumentTypeGenerator.live,
      DocumentStepGenerator.live,
      EvaluationRuntime.live,
      EvaluationContext.live
    )
}
