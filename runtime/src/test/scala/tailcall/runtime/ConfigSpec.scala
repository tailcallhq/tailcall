package tailcall.runtime

import caliban.CalibanError
import tailcall.runtime.dsl.Config
import tailcall.runtime.http.HttpClient
import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service._
import zio.http.Client
import zio.test.Assertion.equalTo
import zio.test.TestAspect.timeout
import zio.test.{ZIOSpecDefault, assertTrue, assertZIO}
import zio.{ZIO, durationInt}

object ConfigSpec extends ZIOSpecDefault {

  def execute(
    config: Config
  )(query: String): ZIO[HttpDataLoader with GraphQLGenerator, CalibanError.ValidationError, String] =
    for {
      graphQL     <- config.toBlueprint.toGraphQL
      interpreter <- graphQL.interpreter
      response    <- interpreter.execute(query)
    } yield response.data.toString

  override def spec =
    suite("ConfigSpec")(
      test("encoding") {
        val extension = DSLFormat.YML
        val config    = JsonPlaceholderConfig.config
        for {
          encoded <- extension.encode(config)
          decoded <- extension.decode(encoded)
        } yield assertTrue(decoded == config)
      },
      test("schema") {
        val config   = JsonPlaceholderConfig.config
        val expected =
          """|schema {
             |  query: Query
             |}
             |
             |type Address {
             |  geo: Geo
             |  street: String
             |  suite: String
             |  city: String
             |  zipcode: String
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
             |type Post {
             |  body: String
             |  id: Int!
             |  user: User @steps(value: [{objectPath: {userId: ["value","userId"]}},{http: {path: "/users/{{userId}}"}}])
             |  userId: Int!
             |  title: String
             |}
             |
             |type Query {
             |  posts: [Post] @steps(value: [{http: {path: "/posts"}}])
             |  users: [User] @steps(value: [{http: {path: "/users"}}])
             |  post(id: Int!): Post @steps(value: [{http: {path: "/posts/{{args.id}}"}}])
             |  user(id: Int!): User @steps(value: [{objectPath: {userId: ["args","id"]}},{http: {path: "/users/{{userId}}"}}])
             |}
             |
             |type User {
             |  website: String
             |  name: String!
             |  posts: [Post] @steps(value: [{http: {path: "/users/{{value.id}}/posts"}}])
             |  email: String!
             |  username: String!
             |  company: Company
             |  id: Int!
             |  address: Address
             |  phone: String
             |}
             |""".stripMargin.trim

        config.toBlueprint.toGraphQL.map(graphQL => assertTrue(graphQL.render == expected))
      },
      suite("execute")(
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
      ),
    ).provide(
      GraphQLGenerator.live,
      StepGenerator.live,
      EvaluationRuntime.default,
      HttpClient.live,
      Client.default,
      DataLoader.http,
    ) @@ timeout(10 seconds)
}
