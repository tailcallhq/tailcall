package tailcall.runtime.transcoder

import tailcall.runtime.JsonT
import tailcall.runtime.model.Config.{Field, Type}
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.{Config, ConfigFormat, Path}
import zio.ZIO
import zio.json.yaml.EncoderYamlOps
import zio.test.Assertion.equalTo
import zio.test.TestAspect.{failing, ignore}
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
            .Type("foo" -> Config.Field.str.withArguments("input" -> Config.Arg.ofType("Foo").withName("data"))),
          "Foo"   -> Config.Type("bar" -> Config.Field.str),
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
      test("batched") {
        val graphQL = s"""
                         |schema {
                         |  query: Query
                         |}
                         |
                         |type Post {
                         |  id: Int
                         |  user: User @http(path: "/users",query: {id: "{{value.userId}}"})
                         |  userId: Int
                         |}
                         |
                         |type Query {
                         |  posts: [Post]
                         |}
                         |
                         |type User {
                         |  id: Int
                         |  name: String
                         |}
                         |""".stripMargin.trim

        val config = Config.default.withTypes(
          "Query" -> Type("posts" -> Field.ofType("Post").asList),
          "User"  -> Type("id" -> Field.int, "name" -> Field.str),
          "Post"  -> Type(
            "id"     -> Field.int,
            "userId" -> Field.int,
            "user"   -> Field.ofType("User")
              .withHttp(path = Path.unsafe.fromString("/users"), query = Map("id" -> "{{value.userId}}")),
          ),
        )

        assertIdentity(config.compress, graphQL)
      },
      test("invalid directive on field") {
        val graphQL = """
                        |type Query {
                        |  foo: String @fooBar
                        |}
                        |""".stripMargin

        val expected = "Cause([Query, foo]: has an unrecognized directive: @fooBar)"
        assertZIO(ConfigFormat.GRAPHQL.decode(graphQL).flip)(equalTo(expected))

        // Config will need to have support for keeping a copy of all the directives.
        // Currently we lose them when we parse a doc into a Config.
      } @@ ignore,
    )

  private def assertIdentity(config: Config, sdl: String): ZIO[Any, String, TestResult] = {
    val configCompressed = config.compress
    for {
      config2SDL <- Transcoder.toSDL(configCompressed, true).toZIO.mapError(_.mkString(", "))
      sdl2Config <- ConfigFormat.GRAPHQL.decode(sdl)
    } yield assertTrue(config2SDL == sdl, sdl2Config.toYaml() == configCompressed.toYaml())
  }
}
