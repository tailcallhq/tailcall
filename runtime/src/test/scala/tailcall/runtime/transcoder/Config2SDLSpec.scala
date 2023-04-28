package tailcall.runtime.transcoder

import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.model.Config
import tailcall.runtime.model.Config.{Arg, Field, Type}
import tailcall.runtime.service._
import zio.test.TestAspect.{failing, timeout}
import zio.test.{TestResult, ZIOSpecDefault, assertTrue}
import zio.{ZIO, durationInt}

/**
 * Tests for the generation of GraphQL schema from a config.
 * This is done by writing a test config, converting to
 * graphql, rendering the generated and then comparing with
 * expected output.
 */
object Config2SDLSpec extends ZIOSpecDefault {
  override def spec =
    suite("Config to SDL")(
      test("only query") {
        val config   = Config.default.withTypes("Query" -> Type("hello" -> Field.ofType("String")))
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  hello: String
                          |}
                          |""".stripMargin.trim
        assertSDL(config, expected)
      },
      test("multiple query") {
        val config   = Config.default.withTypes("Query" -> Type("foo" -> Field.ofType("String")))
          .withTypes("Query" -> Type("bar" -> Field.ofType("String")))
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  foo: String
                          |  bar: String
                          |}
                          |""".stripMargin.trim
        assertSDL(config, expected)
      },
      test("shared input and output types") {
        val config   = Config.default
          .withTypes("Query" -> Type("foo" -> Field.ofType("Foo").withArguments("input" -> Arg.ofType("Foo"))))
          .withTypes("Foo" -> Type("bar" -> Field.ofType("String")))
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |input FooInput {
                          |  bar: String
                          |}
                          |
                          |type Foo {
                          |  bar: String
                          |}
                          |
                          |type Query {
                          |  foo(input: FooInput): Foo
                          |}
                          |""".stripMargin.trim

        assertSDL(config, expected)
      },
      test("shared nested input and output types") {
        val config   = Config.default.withTypes(
          "Query" -> Type("foo" -> Field.ofType("Foo").withArguments("input" -> Arg.ofType("Foo"))),
          "Foo"   -> Type("bar" -> Field.ofType("Bar")),
          "Bar"   -> Type("baz" -> Field.ofType("String")),
        )
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |input BarInput {
                          |  baz: String
                          |}
                          |
                          |input FooInput {
                          |  bar: BarInput
                          |}
                          |
                          |type Bar {
                          |  baz: String
                          |}
                          |
                          |type Foo {
                          |  bar: Bar
                          |}
                          |
                          |type Query {
                          |  foo(input: FooInput): Foo
                          |}
                          |""".stripMargin.trim

        assertSDL(config, expected)
      },
      test("input and output types") {
        val config   = Config.default
          .withTypes("Query" -> Type("foo" -> Field.ofType("Foo").withArguments("input" -> Arg.ofType("FooInput"))))
          .withTypes("Foo" -> Type("bar" -> Field.ofType("String")))
          .withTypes("FooInput" -> Type("bar" -> Field.ofType("String")))
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |input FooInput {
                          |  bar: String
                          |}
                          |
                          |type Foo {
                          |  bar: String
                          |}
                          |
                          |type Query {
                          |  foo(input: FooInput): Foo
                          |}
                          |""".stripMargin.trim

        assertSDL(config, expected)
      },
      test("mergeRight") {
        val config1 = Config.default.withTypes("Query" -> Type("foo" -> Field.ofType("String")))
        val config2 = Config.default.withTypes("Query" -> Type("bar" -> Field.ofType("String")))

        val config   = config1 mergeRight config2
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  foo: String
                          |  bar: String
                          |}
                          |""".stripMargin.trim
        assertSDL(config, expected)
      },
      suite("rename annotations")(
        test("field") {
          val config   = Config.default.withTypes("Query" -> Type("foo" -> Field.ofType("String").withName("bar")))
          val expected = """|schema {
                            |  query: Query
                            |}
                            |
                            |type Query {
                            |  bar: String
                            |}
                            |""".stripMargin.trim
          assertSDL(config, expected)
        },
        test("argument") {
          val config   = Config.default.withTypes(
            "Query" -> Type(
              "foo" -> Field.ofType("String").withArguments("input" -> Arg.ofType("Int").withName("data"))
            )
          )
          val expected = """|schema {
                            |  query: Query
                            |}
                            |
                            |type Query {
                            |  foo(data: Int): String
                            |}
                            |""".stripMargin.trim
          assertSDL(config, expected)
        },
        test("field in input type") {
          val config   = Config.default.withTypes(
            "Query" -> Type("foo" -> Field.ofType("Int").withArguments("input" -> Arg.ofType("Foo"))),
            "Foo"   -> Type("bar" -> Field.ofType("String").withName("baz")),
          )
          val expected = """|schema {
                            |  query: Query
                            |}
                            |
                            |input Foo {
                            |  bar: String
                            |}
                            |
                            |type Query {
                            |  foo(input: Foo): Int
                            |}
                            |""".stripMargin.trim
          assertSDL(config, expected)
        },
      ),

      // TODO: move to jsonplaceholder.spec
      test("json placeholder") {
        val config   = JsonPlaceholderConfig.config
        val expected = """|schema {
                          |  query: Query
                          |  mutation: Mutation
                          |}
                          |
                          |input NewAddress {
                          |  geo: NewGeo
                          |  street: String
                          |  suite: String
                          |  city: String
                          |  zipcode: String
                          |}
                          |
                          |input NewCompany {
                          |  name: String
                          |  catchPhrase: String
                          |  bs: String
                          |}
                          |
                          |input NewGeo {
                          |  lat: String
                          |  lng: String
                          |}
                          |
                          |"A new user."
                          |input NewUser {
                          |  website: String
                          |  name: String!
                          |  email: String!
                          |  username: String!
                          |  company: NewCompany
                          |  address: NewAddress
                          |  phone: String
                          |}
                          |
                          |type Address {
                          |  geo: Geo
                          |  street: String
                          |  suite: String
                          |  city: String
                          |  zip: String
                          |}
                          |
                          |type Company {
                          |  name: String
                          |  catchPhrase: String
                          |  bs: String
                          |}
                          |
                          |type Geo {
                          |  lat: String
                          |  lng: String
                          |}
                          |
                          |"An Id container."
                          |type Id {
                          |  id: Int!
                          |}
                          |
                          |type Mutation {
                          |  createUser("User as an argument." user: NewUser!): Id
                          |}
                          |
                          |type Post {
                          |  body: String
                          |  id: Int!
                          |  user: User
                          |  userId: Int!
                          |  title: String
                          |}
                          |
                          |type Query {
                          |  "A single post by id."
                          |  post(id: Int!): Post
                          |  "A list of all posts."
                          |  posts: [Post]
                          |  "A list of all users."
                          |  users: [User]
                          |  "A single user by id."
                          |  user(id: Int!): User
                          |}
                          |
                          |type User {
                          |  website: String
                          |  name: String!
                          |  posts: [Post]
                          |  email: String!
                          |  username: String!
                          |  company: Company
                          |  id: Int!
                          |  address: Address
                          |  phone: String
                          |}
                          |""".stripMargin.trim

        assertSDL(config, expected)
      },
      test("document type generation") {
        val config = Config.default
          .withTypes("Query" -> Config.Type("test" -> Config.Field.ofType("String").resolveWith("test")))

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test: String
                          |}""".stripMargin
        assertSDL(config, expected)
      },
      test("document with InputValue") {
        val config = Config.default.withTypes(
          "Query" -> Config.Type(
            "test" -> Config.Field.ofType("String").resolveWith("test")
              .withArguments("arg" -> Arg.ofType("String").withDefault("test"))
          )
        )

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertSDL(config, expected)
      },
      test("blueprint with InputValue and default") {
        val config = Config.default.withTypes(
          "Query" -> Config.Type(
            "test" -> Config.Field.ofType("String").resolveWith("test")
              .withArguments("arg" -> Arg.ofType("String").withDefault("test"))
          )
        )

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertSDL(config, expected)
      },
      test("with nesting") {
        val config   = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo"   -> Config.Type("bar" -> Config.Field.ofType("Bar")),
          "Bar"   -> Config.Type("value" -> Config.Field.ofType("Int").resolveWith(100)),
        )
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Bar {
                          |  value: Int
                          |}
                          |
                          |type Foo {
                          |  bar: Bar
                          |}
                          |
                          |type Query {
                          |  foo: Foo
                          |}""".stripMargin
        assertSDL(config, expected)
      },
      test("with nesting array") {
        val config   = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo"   -> Config.Type("bar" -> Config.Field.ofType("Bar").asList),
          "Bar"   -> Config.Type("value" -> Config.Field.ofType("Int")),
        )
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Bar {
                          |  value: Int
                          |}
                          |
                          |type Foo {
                          |  bar: [Bar]
                          |}
                          |
                          |type Query {
                          |  foo: Foo
                          |}""".stripMargin
        assertSDL(config, expected)
      },
      suite("mutation")(
        test("mutation with primitive input") {
          val config = Config.default.withMutation("Mutation").withTypes(
            "Query"    -> Config.Type("foo" -> Config.Field.ofType("Foo").resolveWith(Map("a" -> 1))),
            "Foo"      -> Config.Type("a" -> Config.Field.ofType("Int")),
            "Mutation" -> Config
              .Type("createFoo" -> Config.Field.ofType("Foo").withArguments("input" -> Arg.ofType("String"))),
          )

          val expected = """|schema {
                            |  query: Query
                            |  mutation: Mutation
                            |}
                            |
                            |type Foo {
                            |  a: Int
                            |}
                            |
                            |type Mutation {
                            |  createFoo(input: String): Foo
                            |}
                            |
                            |type Query {
                            |  foo: Foo
                            |}""".stripMargin.trim
          assertSDL(config, expected)
        },
        test("mutation with input type") {
          val config = Config.default.withMutation("Mutation").withTypes(
            "Query"    -> Config.Type.empty,
            "Mutation" -> Config
              .Type("createFoo" -> Config.Field.ofType("Foo").withArguments("input" -> Arg.ofType("FooInput"))),
            "Foo"      -> Config.Type("a" -> Config.Field.ofType("Int")),
            "FooInput" -> Config.Type("a" -> Config.Field.ofType("Int")),
          )

          val expected = """|schema {
                            |  query: Query
                            |  mutation: Mutation
                            |}
                            |
                            |input FooInput {
                            |  a: Int
                            |}
                            |
                            |type Foo {
                            |  a: Int
                            |}
                            |
                            |type Mutation {
                            |  createFoo(input: FooInput): Foo
                            |}
                            |
                            |type Query""".stripMargin.trim
          assertSDL(config, expected)
        },
      ),
      test("omit field") {
        val config = Config.default
          .withTypes("Query" -> Config.Type("foo" -> Config.Field.string, "bar" -> Config.Field.string.withOmit(true)))

        val expected = """
                         |schema {
                         |  query: Query
                         |}
                         |
                         |type Query {
                         |  foo: String
                         |}
                         |""".stripMargin.trim

        assertSDL(config, expected)
      },
      test("inline field") {
        val config = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo").withInline("a", "b")),
          "Foo"   -> Config.Type("a" -> Config.Field.ofType("A")),
          "A"     -> Config.Type("b" -> Config.Field.ofType("B")),
          "B"     -> Config.Type("c" -> Config.Field.ofType("String")),
        )

        val expected = """schema {
                         |  query: Query
                         |}
                         |
                         |type B {
                         |  c: String
                         |}
                         |
                         |type Query {
                         |  foo: B
                         |}
                         |""".stripMargin.trim

        assertSDL(config, expected)
      },
      test("inline field with lists") {
        val config = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo").withInline("a", "b")),
          "Foo"   -> Config.Type("a" -> Config.Field.ofType("A").asList),
          "A"     -> Config.Type("b" -> Config.Field.ofType("B").asList),
          "B"     -> Config.Type("c" -> Config.Field.ofType("String")),
        )

        val expected = """schema {
                         |  query: Query
                         |}
                         |
                         |type B {
                         |  c: String
                         |}
                         |
                         |type Query {
                         |  foo: [[B]]
                         |}
                         |""".stripMargin.trim

        assertSDL(config, expected)
      },
      test("inline on index with list") {
        val config = Config.default.withTypes(
          "Query" -> Config.Type(
            "foo" -> Config.Field.ofType("Foo").withInline("a", "0", "b")
              .resolveWith(Map("a" -> List(Map("b" -> List(Map("c" -> "Hello!"))))))
          ),
          "Foo"   -> Config.Type("a" -> Config.Field.ofType("A").asList),
          "A"     -> Config.Type("b" -> Config.Field.ofType("B").asList),
          "B"     -> Config.Type("c" -> Config.Field.ofType("String")),
        )

        val expected = """schema {
                         |  query: Query
                         |}
                         |
                         |type B {
                         |  c: String
                         |}
                         |
                         |type Query {
                         |  foo: [B]
                         |}
                         |""".stripMargin.trim

        assertSDL(config, expected)
      },
      suite("config generation")(
        test("inline field as config SDL") {
          val config = Config.default.withTypes(
            "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo").withInline("a", "b")),
            "Foo"   -> Config.Type("a" -> Config.Field.ofType("A")),
            "A"     -> Config.Type("b" -> Config.Field.ofType("B")),
            "B"     -> Config.Type("c" -> Config.Field.ofType("String")),
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
                           |}""".stripMargin.trim

          assertSDL(config, expected, true)
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

          assertSDL(config, expected, true)
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

          assertSDL(config, expected, true)

          // TODO: Remove failing after this
          // https://github.com/ghostdogpr/caliban/pull/1690
        } @@ failing,
      ),
    ).provide(GraphQLGenerator.default) @@ timeout(10 seconds)

  private def assertSDL(
    config: Config,
    expected: String,
    asConfig: Boolean = false,
  ): ZIO[GraphQLGenerator, Throwable, TestResult] =
    for { actual <- Transcoder.toSDL(config, asConfig).toTask } yield assertTrue(actual == expected)
}
