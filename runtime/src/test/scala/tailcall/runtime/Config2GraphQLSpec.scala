package tailcall.runtime

import caliban.CalibanError
import tailcall.runtime.http.HttpClient
import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.model.Config.{Arg, Field, Type}
import tailcall.runtime.model.{Config, Step}
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service._
import zio.json.ast.Json
import zio.test.Assertion.equalTo
import zio.test.TestAspect.timeout
import zio.test.{ZIOSpecDefault, assertZIO}
import zio.{Cause, ZIO, durationInt}

object Config2GraphQLSpec extends ZIOSpecDefault {

  def execute(
    config: Config
  )(query: String): ZIO[HttpDataLoader with GraphQLGenerator, CalibanError.ValidationError, String] = {
    for {
      graphQL     <- config.toBlueprint.toGraphQL
      interpreter <- graphQL.interpreter
      response    <- interpreter.execute(query)
      _ <- ZIO.foreachDiscard(response.errors)(error => ZIO.logErrorCause("GraphQL Execution Error", Cause.fail(error)))
    } yield response.data.toString
  }

  override def spec =
    suite("config to graphql")(
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
            .withType("Query" -> Type("foo" -> Field.ofType("String").resolveWith("Hello World!").withName("bar")))
        }
        val program = execute(config)(""" query { bar } """)

        assertZIO(program)(equalTo("""{"bar":"Hello World!"}"""))
      },
      test("rename an argument") {
        val config  = {
          Config.default.withType(
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

        val config = Config.default.withType(
          "Query" -> Type("a" -> Field.ofType("A").withSteps(Step.constant(value))),
          "A"     -> Type("b" -> Field.ofType("B").asList),
          "B"     -> Type("c" -> Field.int),
        )

        val program = execute(config)("""{a {b {c}}}""")
        assertZIO(program)(equalTo("""{"a":{"b":[{"c":1},{"c":2},{"c":3}]}}"""))
      },
      test("dictionary") {
        val value  = Json.Obj(
          "a" -> Json.Num(1), //
          "b" -> Json.Obj(
            //
            "k1" -> Json.Num(1),
            "k2" -> Json.Num(2),
            "k3" -> Json.Num(3),
          ),
        )
        val config = Config.default.withType(
          "Query" -> Type(
            "a" -> Field.ofType("A").withSteps(
              //
              Step.constant(value),
              Step.transform(JsonT.applySpec("a" -> JsonT.identity, "b" -> JsonT.toPair)),
            )
          ),
          "A"     -> Type("a" -> Field.int, "b" -> Field.ofType("B")),
          "B"     -> Type("key" -> Field.string, "value" -> Field.int),
        )

        pprint.pprintln(value.toJson)
        val program = execute(config)("""{a {b {key, value}}}""")
        assertZIO(program)(equalTo(
          """{"a":{"b":[{"key":"c","value":1},{"key":"d","value":2},{"key":"e","value":3}]}}"""
        ))
      },
    ).provide(GraphQLGenerator.default, HttpClient.default, DataLoader.http) @@ timeout(10 seconds)
}
