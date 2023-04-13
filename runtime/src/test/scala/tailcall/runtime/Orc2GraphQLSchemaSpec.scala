package tailcall.runtime

import tailcall.runtime.model.Config
import tailcall.runtime.model.Config.Arg
import tailcall.runtime.service._
import zio.ZIO
import zio.test.Assertion._
import zio.test._

object Orc2GraphQLSchemaSpec extends ZIOSpecDefault {
  override def spec =
    suite("config to graphql schema")(
      test("document type generation") {
        val config = Config.default
          .withTypes("Query" -> Config.Type("test" -> Config.Field.ofType("String").resolveWithDynamicValue("test")))

        val actual   = render(config)
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
        val config = Config.default.withTypes(
          "Query" -> Config.Type(
            "test" -> Config.Field.ofType("String").resolveWithDynamicValue("test")
              .withArguments("arg" -> Arg.ofType("String").withDefault("test"))
          )
        )
        val actual = render(config)

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      },
      test("blueprint with InputValue and default") {
        val config = Config.default.withTypes(
          "Query" -> Config.Type(
            "test" -> Config.Field.ofType("String").resolveWithDynamicValue("test")
              .withArguments("arg" -> Arg.ofType("String").withDefault("test"))
          )
        )
        val actual = render(config)

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      },
      test("with nesting") {
        val config   = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo"   -> Config.Type("bar" -> Config.Field.ofType("Bar")),
          "Bar"   -> Config.Type("value" -> Config.Field.ofType("Int").resolveWithDynamicValue(100)),
        )
        val schema   = render(config)
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
        assertZIO(schema)(equalTo(expected))
      },
      test("with nesting array") {
        val config   = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo"   -> Config.Type("bar" -> Config.Field.ofType("Bar").asList),
          "Bar"   -> Config.Type("value" -> Config.Field.ofType("Int")),
        )
        val schema   = render(config)
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
        assertZIO(schema)(equalTo(expected))
      },
      suite("mutation")(
        test("mutation with primitive input") {
          // mutation createFoo(input: String){foo: Foo}
          // type Foo {a: Int, b: Int, c: Int}
          val config = Config.default.withMutation("Mutation").withTypes(
            "Query"    -> Config.Type("foo" -> Config.Field.ofType("Foo").resolveWithDynamicValue(Map("a" -> 1))),
            "Foo"      -> Config.Type("a" -> Config.Field.ofType("Int")),
            "Mutation" -> Config
              .Type("createFoo" -> Config.Field.ofType("Foo").withArguments("input" -> Arg.ofType("String"))),
          )

          val schema = render(config)
          assertZIO(schema)(equalTo("""|schema {
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
                                       |}""".stripMargin))
        },
        test("mutation with input type") {
          // schema {mutation: Mutation}
          // type Mutation { createFoo(input: FooInput) Foo }
          // type Foo { foo: String }
          // input FooInput {a: Int, b: Int, c: Int}

          val config = Config.default.withMutation("Mutation").withTypes(
            "Query"    -> Config.Type.empty,
            "Mutation" -> Config
              .Type("createFoo" -> Config.Field.ofType("Foo").withArguments("input" -> Arg.ofType("FooInput"))),
            "Foo"      -> Config.Type("a" -> Config.Field.ofType("Int")),
            "FooInput" -> Config.Type("a" -> Config.Field.ofType("Int")),
          )

          val schema = config.toBlueprint.toGraphQL.map(_.render)
          assertZIO(schema)(equalTo("""|schema {
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
                                       |type Query""".stripMargin))
        },
      ),
    ).provide(GraphQLGenerator.live, StepGenerator.live, EvaluationRuntime.default)

  def render(config: Config): ZIO[GraphQLGenerator, Throwable, String] = config.toBlueprint.toGraphQL.map(_.render)
}
