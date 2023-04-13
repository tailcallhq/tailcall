package tailcall.runtime

import caliban.{CalibanError, InputValue}
import tailcall.runtime.http.HttpClient
import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.model.Config.{Arg, Field, Type}
import tailcall.runtime.model.{Blueprint, Config, Step}
import tailcall.runtime.remote._
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service._
import zio.json.ast.Json
import zio.test.Assertion.equalTo
import zio.test.TestAspect.timeout
import zio.test.{ZIOSpecDefault, assertZIO}
import zio.{Cause, ZIO, durationInt}

/**
 * Tests for the generation of GraphQL steps from a config.
 * This is done by writing a test config, converting to
 * graphql and testing it with sample graphql queries.
 */
object GraphQLStepGenerationSpec extends ZIOSpecDefault {
  override def spec =
    suite("GraphQL Step Generation")(
      test("users name") {
        val program = execute(JsonPlaceholderConfig.config)(""" query { users {name} } """)

        val expected = """{"users":[
                         |{"name":"Leanne Graham"},
                         |{"name":"Ervin Howell"},
                         |{"name":"Clementine Bauch"},
                         |{"name":"Patricia Lebsack"},
                         |{"name":"Chelsey Dietrich"},
                         |{"name":"Mrs. Dennis Schulist"},
                         |{"name":"Kurtis Weissnat"},
                         |{"name":"Nicholas Runolfsdottir V"},
                         |{"name":"Glenna Reichert"},
                         |{"name":"Clementina DuBuque"}
                         |]}""".stripMargin.replace("\n", "").trim
        assertZIO(program)(equalTo(expected))
      },
      test("user name") {
        val program = execute(JsonPlaceholderConfig.config)(""" query { user(id: 1) {name} } """)
        assertZIO(program)(equalTo("""{"user":{"name":"Leanne Graham"}}"""))
      },
      test("post body") {
        val program  = execute(JsonPlaceholderConfig.config)(""" query { post(id: 1) { title } } """)
        val expected =
          """{"post":{"title":"sunt aut facere repellat provident occaecati excepturi optio reprehenderit"}}"""
        assertZIO(program)(equalTo(expected))
      },
      test("user company") {
        val program  = execute(JsonPlaceholderConfig.config)(""" query {user(id: 1) { company { name } } }""")
        val expected = """{"user":{"company":{"name":"Romaguera-Crona"}}}"""
        assertZIO(program)(equalTo(expected))
      },
      test("user posts") {
        val program  = execute(JsonPlaceholderConfig.config)(""" query {user(id: 1) { posts { title } } }""")
        val expected =
          """{"user":{"posts":[{"title":"sunt aut facere repellat provident occaecati excepturi optio reprehenderit"},
            |{"title":"qui est esse"},
            |{"title":"ea molestias quasi exercitationem repellat qui ipsa sit aut"},
            |{"title":"eum et est occaecati"},
            |{"title":"nesciunt quas odio"},
            |{"title":"dolorem eum magni eos aperiam quia"},
            |{"title":"magnam facilis autem"},
            |{"title":"dolorem dolore est ipsam"},
            |{"title":"nesciunt iure omnis dolorem tempora et accusantium"},
            |{"title":"optio molestias id quia eum"}]}}""".stripMargin.replace("\n", "").trim
        assertZIO(program)(equalTo(expected))
      },
      test("post user") {
        val program  = execute(JsonPlaceholderConfig.config)(""" query {post(id: 1) { title user { name } } }""")
        val expected =
          """{"post":{"title":"sunt aut facere repellat provident occaecati excepturi optio reprehenderit","user":{"name":"Leanne Graham"}}}"""
        assertZIO(program)(equalTo(expected))
      },
      test("create user") {
        val program = execute(JsonPlaceholderConfig.config)(
          """ mutation { createUser(user: {name: "test", email: "test@abc.com", username: "test"}) { id } } """
        )
        assertZIO(program)(equalTo("""{"createUser":{"id":11}}"""))
      },
      test("create user with zip code") {
        val program = execute(JsonPlaceholderConfig.config)(
          """ mutation { createUser(user: {name: "test", email: "test@abc.com", username: "test", address: {zip: "1234-4321"}}) { id } } """
        )
        assertZIO(program)(equalTo("""{"createUser":{"id":11}}"""))
      },
      test("rename a field") {
        val config  = {
          Config.default
            .withTypes("Query" -> Type("foo" -> Field.ofType("String").resolveWith("Hello World!").withName("bar")))
        }
        val program = execute(config)(""" query { bar } """)

        assertZIO(program)(equalTo("""{"bar":"Hello World!"}"""))
      },
      test("rename an argument") {
        val config  = {
          Config.default.withTypes(
            "Query" -> Type(
              "foo" -> Field.ofType("Bar").withArguments("input" -> Arg.ofType("Int").withName("data"))
                .withSteps(Step.objPath("bar" -> List("args", "data")))
            ),
            "Bar"   -> Type("bar" -> Field.ofType("Int")),
          )
        }
        val program = execute(config)(""" query { foo(data: 1) {bar} } """)

        assertZIO(program)(equalTo("""{"foo":{"bar":1}}"""))
      },
      test("user zipcode") {
        val program  = execute(JsonPlaceholderConfig.config)("""query { user(id: 1) { address { zip } } }""")
        val expected = """{"user":{"address":{"zip":"92998-3874"}}}"""
        assertZIO(program)(equalTo(expected))
      },
      test("nested type") {
        val value = Json.Obj(
          "b" -> Json.Arr(
            //
            Json.Obj("c" -> Json.Num(1)),
            Json.Obj("c" -> Json.Num(2)),
            Json.Obj("c" -> Json.Num(3)),
          )
        )

        val config = Config.default.withTypes(
          "Query" -> Type("a" -> Field.ofType("A").withSteps(Step.constant(value))),
          "A"     -> Type("b" -> Field.ofType("B").asList),
          "B"     -> Type("c" -> Field.int),
        )

        val program = execute(config)("""{a {b {c}}}""")
        assertZIO(program)(equalTo("""{"a":{"b":[{"c":1},{"c":2},{"c":3}]}}"""))
      },
      test("dictionary") {
        val value: Json = Json
          .Obj("a" -> Json.Num(1), "b" -> Json.Obj("k1" -> Json.Num(1), "k2" -> Json.Num(2), "k3" -> Json.Num(3)))

        val transformation = JsonT.applySpec("a" -> JsonT.path("a"), "b" -> JsonT.path("b").andThen(JsonT.toKeyValue))

        val config = Config.default.withTypes(
          "Query" -> Type("z" -> Field.ofType("A").withSteps(Step.constant(value), Step.transform(transformation))),
          "A"     -> Type("a" -> Field.int, "b" -> Field.ofType("B").asList),
          "B"     -> Type("key" -> Field.string, "value" -> Field.int),
        )

        val program = execute(config)("""{z {a b {key value}}}""")
        assertZIO(program)(equalTo(
          """{"z":{"a":1,"b":[{"key":"k1","value":1},{"key":"k2","value":2},{"key":"k3","value":3}]}}"""
        ))
      },
      test("simple query") {
        val config  = Config.default
          .withTypes("Query" -> Type("foo" -> Field.ofType("String").resolveWith("Hello World!")))
        val program = execute(config)(" {foo} ")
        assertZIO(program)(equalTo("""{"foo":"Hello World!"}"""))
      },
      test("nested objects") {
        val config = Config.default.withTypes(
          "Query" -> Type("foo" -> Field.ofType("Foo").resolveWith(Map("bar" -> "Hello World!"))),
          "Foo"   -> Type("bar" -> Field.ofType("String")),
        )

        val program = execute(config)(" {foo {bar}} ")
        assertZIO(program)(equalTo("""{"foo":{"bar":"Hello World!"}}"""))
      },
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
    ).provide(GraphQLGenerator.default, HttpClient.default, DataLoader.http) @@ timeout(10 seconds)

  private def execute(
    config: Config
  )(query: String): ZIO[HttpDataLoader with GraphQLGenerator, CalibanError.ValidationError, String] = {
    for {
      graphQL     <- config.toBlueprint.toGraphQL
      interpreter <- graphQL.interpreter
      response    <- interpreter.execute(query)
      _ <- ZIO.foreachDiscard(response.errors)(error => ZIO.logErrorCause("GraphQL Execution Error", Cause.fail(error)))
    } yield response.data.toString
  }

  private def execute(config: Config, variables: Map[String, InputValue])(
    query: String
  ): ZIO[HttpDataLoader with GraphQLGenerator, Throwable, String] = execute(config.toBlueprint, variables)(query)

  private def execute(doc: Blueprint, variables: Map[String, InputValue])(
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
