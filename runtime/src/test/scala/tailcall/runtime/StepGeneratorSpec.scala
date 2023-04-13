package tailcall.runtime

import caliban.{CalibanError, InputValue}
import tailcall.runtime.http.HttpClient
import tailcall.runtime.model.{Blueprint, Config}
import tailcall.runtime.remote._
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service._
import zio.ZIO
import zio.http.Client
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

object StepGeneratorSpec extends ZIOSpecDefault {

  def spec = {
    suite("StepGenerator")(
      test("static value") {
        val config  = Config.default
          .withTypes("Query" -> Config.Type("id" -> Config.Field.ofType("String").resolveWithDynamicValue(100)))
        val program = execute(config)("query {id}")
        assertZIO(program)(equalTo("""{"id":100}"""))
      },
      test("with args") {
        val config  = Config.default.withTypes(
          "Query" -> Config.Type(
            "sum" -> Config.Field.ofType("Int")
              .withArguments("a" -> Config.Arg.ofType("Int"), "b" -> Config.Arg.ofType("Int"))
              .resolveWithFunction { ctx =>
                {
                  (for {
                    a <- ctx.toTypedPath[Int]("args", "a")
                    b <- ctx.toTypedPath[Int]("args", "b")
                  } yield a + b).toDynamic
                }
              }
          )
        )
        val program = execute(config)("query {sum(a: 1, b: 2)}")
        assertZIO(program)(equalTo("""{"sum":3}"""))
      },
      test("with nesting") {
        // type Query {foo: Foo}
        // type Foo {bar: Bar}
        // type Bar {value: Int}

        val config = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo"   -> Config.Type("bar" -> Config.Field.ofType("Bar")),
          "Bar"   -> Config.Type("value" -> Config.Field.ofType("Int").resolveWithDynamicValue(100)),
        )

        val program = execute(config)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("{\"foo\":{\"bar\":{\"value\":100}}}"))
      },
      test("with nesting array") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {value: Int}

        val config = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo" -> Config.Type("bar" -> Config.Field.ofType("Bar").asList.resolveWithDynamicValue(List(100, 200, 300))),
          "Bar" -> Config.Type("value" -> Config.Field.ofType("Int").resolveWithDynamicValue(100)),
        )

        val program = execute(config)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("""{"foo":{"bar":[{"value":100},{"value":100},{"value":100}]}}"""))
      },
      test("with nesting array ctx") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {value: Int}
        val config = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo" -> Config.Type("bar" -> Config.Field.ofType("Bar").asList.resolveWithDynamicValue(List(100, 200, 300))),
          "Bar" -> Config.Type("value" -> Config.Field.ofType("Int").resolveWithFunction {
            _.toTypedPath[Int]("value").map(_ + Remote(1)).toDynamic
          }),
        )

        val program = execute(config)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("""{"foo":{"bar":[{"value":101},{"value":201},{"value":301}]}}"""))
      },
      test("with nesting level 3") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {baz: Baz}
        // type Baz{value: Int}
        val config = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo" -> Config.Type("bar" -> Config.Field.ofType("Bar").asList.resolveWithDynamicValue(List(100, 200, 300))),
          "Bar" -> Config.Type("baz" -> Config.Field.ofType("Baz").resolveWithFunction {
            _.toTypedPath[Int]("value").map(_ + Remote(1)).toDynamic
          }),
          "Baz" -> Config.Type("value" -> Config.Field.ofType("Int").resolveWithFunction {
            _.toTypedPath[Option[Int]]("value").flatten.map(_ + Remote(1)).toDynamic
          }),
        )

        val program = execute(config)("query {foo { bar { baz {value} }}}")
        assertZIO(program)(equalTo(
          """{"foo":{"bar":[{"baz":{"value":102}},{"baz":{"value":202}},{"baz":{"value":302}}]}}"""
        ))
      },
      test("parent") {
        // type Query {foo: Foo}
        // type Foo {bar: Bar}
        // type Bar{baz: Baz}
        // type Baz{value: Int}
        val config  = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Foo"   -> Config.Type("bar" -> Config.Field.ofType("Bar").resolveWithDynamicValue(100)),
          "Bar"   -> Config.Type("baz" -> Config.Field.ofType("Baz").resolveWithDynamicValue(200)),
          "Baz"   -> Config.Type("value" -> Config.Field.ofType("Int").resolveWithFunction {
            _.toTypedPath[Int]("parent", "value").toDynamic
          }),
        )
        val program = execute(config)("query {foo { bar { baz {value} }}}")
        assertZIO(program)(equalTo("""{"foo":{"bar":{"baz":{"value":100}}}}"""))

      },
      test("partial resolver") {
        // type Query {foo: Foo}
        // type Foo {a: Int, b: Int, c: Int}
        val config  = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo").resolveWithDynamicValue(Map("a" -> 1, "b" -> 2))),
          "Foo"   -> Config.Type(
            "a" -> Config.Field.ofType("Int"),
            "b" -> Config.Field.ofType("Int"),
            "c" -> Config.Field.ofType("Int").resolveWithDynamicValue(3),
          ),
        )
        val program = execute(config)("query {foo { a b c }}")
        assertZIO(program)(equalTo("""{"foo":{"a":1,"b":2,"c":3}}"""))

      },
      test("default property resolver") {
        // type Query {foo: Foo}
        // type Foo {a: Int, b: Int, c: Int}
        val config  = Config.default.withTypes(
          "Query" -> Config.Type("foo" -> Config.Field.ofType("Foo").resolveWithDynamicValue(Map("a" -> 1))),
          "Foo"   -> Config.Type("a" -> Config.Field.ofType("Int")),
        )
        val program = execute(config)("query {foo { a }}")
        assertZIO(program)(equalTo("""{"foo":{"a":1}}"""))

      },
      test("mutation with input type") {
        // type Mutation { createFoo(input: FooInput){foo: Foo} }
        // type Foo {a : Int}
        // input FooInput {a: Int, b: Int, c: Int}

        val config = Config.default.withMutation("Mutation").withTypes(
          "Query"    -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Mutation" -> Config.Type(
            "createFoo" -> Config.Field.ofType("Foo").withArguments("input" -> Config.Arg.ofType("FooInput"))
              .resolveWithDynamicValue(Map("a" -> 1))
          ),
          "Foo"      -> Config.Type("a" -> Config.Field.ofType("Int")),
          "FooInput" -> Config.Type(
            "a" -> Config.Field.ofType("Int"),
            "b" -> Config.Field.ofType("Int"),
            "c" -> Config.Field.ofType("Int"),
          ),
        )

        val program = execute(config, Map.empty)("mutation {createFoo(input: {a: 1}){a}}")
        assertZIO(program)(equalTo("""{"createFoo":{"a":1}}"""))
      },
    ).provide(
      GraphQLGenerator.live,
      StepGenerator.live,
      EvaluationRuntime.default,
      HttpClient.live,
      Client.default,
      DataLoader.http,
    )
  }

  def execute(config: Config, variables: Map[String, InputValue] = Map.empty)(
    query: String
  ): ZIO[HttpDataLoader with GraphQLGenerator, Throwable, String] = execute(config.toBlueprint, variables)(query)

  def execute(doc: Blueprint, variables: Map[String, InputValue])(
    query: String
  ): ZIO[HttpDataLoader with GraphQLGenerator, CalibanError, String] =
    for {
      graphQL     <- doc.toGraphQL
      interpreter <- graphQL.interpreter
      result      <- interpreter.execute(query, variables = variables)
      _           <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
}
