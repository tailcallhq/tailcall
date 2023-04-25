package tailcall.runtime

import tailcall.runtime.model.Config.{Field, Type}
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.{Config, Path}
import tailcall.runtime.service.ConfigFormat
import zio.ZIO
import zio.json.EncoderOps
import zio.test._

object ConfigFormatSpec extends ZIOSpecDefault {
  private def assertIdentity(config: Config, graphQL: String): ZIO[Any, String, TestResult] =
    for {
      encodedConfig  <- config.asGraphQLConfig
      decodedGraphQL <- ConfigFormat.GRAPHQL.decode(graphQL)
    } yield assertTrue(encodedConfig == graphQL, decodedGraphQL.toJsonPretty == config.toJsonPretty)

  def spec =
    suite("DSLFormat")(suite("graphql config identity")(
      test("http directive") {
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
          "Query" -> Type(
            "foo" -> Field.ofType("User").asList.withHttp(Operation.Http(Path.unsafe.fromString("/users")))
          ),
          "User"  -> Type("id" -> Field.ofType("Int"), "name" -> Field.ofType("String")),
        )

        assertIdentity(config, graphQL)
      },
      test("unsafe directive") {
        val graphQL = """
                        |schema {
                        |  query: Query
                        |}
                        |
                        |type Query {
                        |  foo: [User] @unsafe(steps: [{http: {path: "/users"}},{transform: {path: ["data","users"]}}])
                        |}
                        |
                        |type User {
                        |  id: Int
                        |  name: String
                        |}
                        |
                        |""".stripMargin.trim

        val config = Config.default.withTypes(
          "Query" -> Type(
            "foo" -> Field.ofType("User").asList.withSteps(
              Operation.Http(Path.unsafe.fromString("/users")),
              Operation.Transform(JsonT.path("data", "users")),
            )
          ),
          "User"  -> Type("id" -> Field.ofType("Int"), "name" -> Field.ofType("String")),
        )

        assertIdentity(config, graphQL)
      },
    ))
}
