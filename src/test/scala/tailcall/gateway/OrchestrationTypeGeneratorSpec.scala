package tailcall.gateway

import tailcall.gateway.ast.Orchestration
import tailcall.gateway.remote.Remote
import tailcall.gateway.service._
import zio.schema.DynamicValue
import zio.test.Assertion._
import zio.test._

object OrchestrationTypeGeneratorSpec extends ZIOSpecDefault {
  override def spec =
    suite("DocumentTypeGenerator")(
      test("document type generation") {
        val document = Orchestration(List(
          Orchestration.ObjectTypeDefinition(
            "Query",
            List(Orchestration.FieldDefinition(
              "test",
              List(),
              Orchestration.NamedType("String", false),
              _ => Remote(DynamicValue("test"))
            ))
          ),
          Orchestration.SchemaDefinition(Some("Query"), None, None)
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
        val document = Orchestration(List(
          Orchestration.ObjectTypeDefinition(
            "Query",
            List(Orchestration.FieldDefinition(
              "test",
              List(
                Orchestration
                  .InputValueDefinition("arg", Orchestration.NamedType("String", false), Some(DynamicValue("test")))
              ),
              Orchestration.NamedType("String", false),
              _ => Remote(DynamicValue("test"))
            ))
          ),
          Orchestration.SchemaDefinition(Some("Query"), None, None)
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
      OrchestrationGraphQLGenerator.live,
      OrchestrationTypeGenerator.live,
      OrchestrationStepGenerator.live,
      EvaluationRuntime.live,
      EvaluationContext.live
    )
}
