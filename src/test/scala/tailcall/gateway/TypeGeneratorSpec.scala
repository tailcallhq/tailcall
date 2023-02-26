package tailcall.gateway

import tailcall.gateway.dsl.scala.Orc
import tailcall.gateway.service._
import zio.test.Assertion._
import zio.test._

object TypeGeneratorSpec extends ZIOSpecDefault {
  import Orc._
  override def spec =
    suite("DocumentTypeGenerator")(
      test("document type generation") {
        val field  = Field.output("test").as("String").resolveWith("test")
        val query  = Obj("Query").withFields(field)
        val orc    = Orc.empty.withQuery("Query").withType(query)
        val actual = orc.toDocument.toGraphQL.map(_.render)

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test: String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      },
      test("document with InputValue") {
        val input  = Field.input("arg").as("String").withDefault("test")
        val field  = Field.output("test").as("String").resolveWith("test").withArgument(input)
        val query  = Obj("Query").withFields(field)
        val orc    = Orc.empty.withQuery("Query").withType(query)
        val actual = orc.toDocument.toGraphQL.map(_.render)

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      }
    ).provide(
      GraphQLGenerator.live,
      TypeGenerator.live,
      StepGenerator.live,
      EvaluationRuntime.live,
      EvaluationContext.live
    )
}
