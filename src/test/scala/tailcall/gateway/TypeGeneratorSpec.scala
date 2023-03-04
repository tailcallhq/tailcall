package tailcall.gateway

import tailcall.gateway.dsl.scala.Orc
import tailcall.gateway.dsl.scala.Orc.Field
import tailcall.gateway.service._
import zio.ZIO
import zio.test.Assertion._
import zio.test._

object TypeGeneratorSpec extends ZIOSpecDefault {
  override def spec =
    suite("DocumentTypeGenerator")(
      test("document type generation") {
        val orc = Orc("Query" -> List("test" -> Field.output.to("String").resolveWith("test")))

        val actual   = render(orc)
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
        val orc    = Orc(
          "Query" -> List(
            "test" -> Field.output.to("String").resolveWith("test")
              .withArgument("arg" -> Field.input.to("String").withDefault("test"))
          )
        )
        val actual = render(orc)

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      }
    ).provide(GraphQLGenerator.live, TypeGenerator.live, StepGenerator.live, EvaluationRuntime.live)

  def render(orc: Orc): ZIO[GraphQLGenerator, Throwable, String] = orc.toBlueprint.flatMap(_.toGraphQL).map(_.render)
}
