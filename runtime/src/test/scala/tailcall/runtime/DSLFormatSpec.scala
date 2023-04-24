package tailcall.runtime

import tailcall.runtime.model.Config.{Field, Type}
import tailcall.runtime.model.{Config, Path, Step}
import tailcall.runtime.service.DSLFormat
import zio.ZIO
import zio.json.EncoderOps
import zio.test._

object DSLFormatSpec extends ZIOSpecDefault {
  private def assertIdentity(config: Config, graphQL: String): ZIO[Any, String, TestResult] =
    for {
      encodedConfig  <- config.asGraphQLConfig
      decodedGraphQL <- DSLFormat.GRAPHQL.decode(graphQL)
    } yield assertTrue(encodedConfig == graphQL, decodedGraphQL.toJsonPretty == config.toJsonPretty)

  def spec =
    suite("DSLFormat")(suite("graphql config identity")(test("http directive") {
      val graphQL = """
                      |schema {
                      |  query: Query
                      |}
                      |
                      |type Query {
                      |  foo: [User] @http(path: "/users")
                      |}
                      |
                      |type User {
                      |  id: Int
                      |  name: String
                      |}
                      |
                      |""".stripMargin.trim

      val config = Config.default.withTypes(
        "Query" -> Type("foo" -> Field.ofType("User").asList.withHttp(Step.Http(Path.unsafe.fromString("/users")))),
        "User"  -> Type("id" -> Field.ofType("Int"), "name" -> Field.ofType("String")),
      )

      assertIdentity(config, graphQL)
    }))
}
