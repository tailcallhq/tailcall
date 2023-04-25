package tailcall.runtime

import tailcall.runtime.model.Config
import zio.test.TestAspect.failing
import zio.test.{Spec, TestEnvironment, TestResult, ZIOSpecDefault, assertTrue}
import zio.{Scope, ZIO}

object Config2SDL0 extends ZIOSpecDefault {

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("DocumentGeneration")(test("input type directives") {
      val config = Config.default.withTypes(
        "Query" -> Config
          .Type("foo" -> Config.Field.string.withArguments("input" -> Config.Arg.ofType("Foo").withName("data"))),
        "Foo"   -> Config.Type("bar" -> Config.Field.string),
      )

      assertSchema(config) {
        """schema {
          |  query: Query
          |}
          |
          |input Foo {
          |  bar: String
          |}
          |
          |type Query {
          |  foo(input: Foo @modify(rename: "data")): String
          |}
          |""".stripMargin
      }

      // TODO: Remove failing after this
      // https://github.com/ghostdogpr/caliban/pull/1690
    } @@ failing)

  private def assertSchema(config: Config)(expected: String): ZIO[Any, String, TestResult] =
    for { graphQL <- config.asGraphQLConfig } yield assertTrue(graphQL == expected.trim)
}
