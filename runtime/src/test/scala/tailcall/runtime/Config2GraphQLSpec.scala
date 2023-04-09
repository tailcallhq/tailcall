package tailcall.runtime

import caliban.CalibanError
import tailcall.runtime.http.HttpClient
import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.model.Config.{Arg, Field, Type}
import tailcall.runtime.model.{Config, Step}
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service._
import zio.json._
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
          Config.empty.withQuery("Query")
            .withType("Query" -> Type("foo" -> Field.ofType("String").resolveWith("Hello World!").withName("bar")))
        }
        val program = execute(config)(""" query { bar } """)

        assertZIO(program)(equalTo("""{"bar":"Hello World!"}"""))
      },
      test("rename an argument") {
        val config  = {
          Config.empty.withQuery("Query").withType(
            "Query" -> Type(
              "foo" -> Field.ofType("Bar").withArguments("input" -> Arg.ofType("Int").withName("data"))
                .withSteps(Step.ObjPath("bar" -> List("args", "data")))
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
        val value  = Map("a" -> "abc".toJsonAST.toOption.get, "b" -> List(Map("bar" -> "bar")).toJsonAST.toOption.get)
          .toJsonAST.toOption.get
        val config = Config.empty.withQuery("Query").withType(
          "Query" -> Type("foo" -> Field.ofType("Foo").withSteps(Step.Constant(value))),
          "Foo"   -> Type("a" -> Field.ofType("String"), "b" -> Field.ofType("Bar").asList),
          "Bar"   -> Type("bar" -> Field.ofType("String")),
        )

        val program = execute(config)("""{foo {a b {bar}}}""")
        assertZIO(program)(equalTo("""{"foo":{"a":"abc","b":[{"bar":"bar"}]}}"""))
      },
    ).provide(GraphQLGenerator.default, HttpClient.default, DataLoader.http) @@ timeout(10 seconds)
}
