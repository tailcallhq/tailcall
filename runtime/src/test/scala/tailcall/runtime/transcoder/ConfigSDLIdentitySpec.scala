package tailcall.runtime.transcoder

import tailcall.runtime.JsonT
import tailcall.runtime.model.Config.{Field, Type}
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.{Config, ConfigFormat, Path}
import zio.ZIO
import zio.json.EncoderOps
import zio.test.TestAspect.failing
import zio.test._

/**
 * Converts a SDL to a Config and then vice-versa, and
 * asserts that the they are equal.
 */
object ConfigSDLIdentitySpec extends ZIOSpecDefault {
  def spec =
    suite("graphql config identity")(
      test("inline field as config SDL") {
        val config = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo").withInline("a", "b")),
          "A"     -> Config.Type("b" -> Config.Field.ofType("B")),
          "B"     -> Config.Type("c" -> Config.Field.ofType("String")),
          "Foo"   -> Config.Type("a" -> Config.Field.ofType("A")),
        )

        val expected = """schema {
                         |  query: Query
                         |}
                         |
                         |type A {
                         |  b: B
                         |}
                         |
                         |type B {
                         |  c: String
                         |}
                         |
                         |type Foo {
                         |  a: A
                         |}
                         |
                         |type Query {
                         |  foo: Foo @inline(path: ["a","b"])
                         |}
                         |""".stripMargin.trim

        assertIdentity(config, expected)
      },
      test("variable in server directives") {
        val config   = Config.default.withVars("foo" -> "bar")
        val expected = """
                         |schema @server(vars: {foo: "bar"}) {
                         |  query: Query
                         |}
                         |
                         |type Query
                         |""".stripMargin.trim

        assertIdentity(config, expected)
      } @@ failing,
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
      test("input type directives") {
        val config = Config.default.withTypes(
          "Query" -> Config
            .Type("foo" -> Config.Field.string.withArguments("input" -> Config.Arg.ofType("Foo").withName("data"))),
          "Foo"   -> Config.Type("bar" -> Config.Field.string),
        )

        val expected = """schema {
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

        assertIdentity(config, expected)

        // TODO: Remove failing after this
        // https://github.com/ghostdogpr/caliban/pull/1690
      } @@ failing,
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
    )

  private def assertIdentity(config: Config, sdl: String): ZIO[Any, String, TestResult] =
    for {
      encodedConfig  <- Transcoder.toSDL(config, true).toZIO.mapError(_.mkString(", "))
      decodedGraphQL <- ConfigFormat.GRAPHQL.decode(sdl)
    } yield assertTrue(encodedConfig == sdl, decodedGraphQL.toJsonAST == config.toJsonAST)
}
