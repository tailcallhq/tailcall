package tailcall.gateway

import tailcall.gateway.dsl.json.Config
import tailcall.gateway.internal.{Extension, JsonPlaceholderConfig}
import tailcall.gateway.service.{EvaluationRuntime, GraphQLGenerator, StepGenerator, TypeGenerator}
import zio.ZIO
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertTrue, assertZIO}

object ConfigSpec extends ZIOSpecDefault {

  def execute(config: Config)(query: String): ZIO[GraphQLGenerator, Throwable, String] =
    for {
      graphQL     <- config.toBlueprint.toGraphQL
      interpreter <- graphQL.interpreter
      response    <- interpreter.execute(query)
    } yield response.data.toString

  override def spec =
    suite("ConfigSpec")(
      test("encoding") {
        val extension = Extension.YML
        val config    = JsonPlaceholderConfig.config
        for {
          encoded <- extension.encode(config)
          decoded <- extension.decode(encoded)
        } yield assertTrue(decoded == config)
      },
      test("schema") {
        val config   = JsonPlaceholderConfig.config
        val expected = """
                         |schema {
                         |  query: Query
                         |}
                         |
                         |scalar ID!
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
                         |  id: ID!
                         |  user: User
                         |  userId: ID!
                         |  title: String
                         |}
                         |
                         |type Query {
                         |  posts: [Post]
                         |  users: [User]
                         |  post(id: ID!): Post
                         |  user(id: ID!): User
                         |}
                         |
                         |type User {
                         |  website: String
                         |  name: String!
                         |  posts: [Post]
                         |  email: String!
                         |  username: String!
                         |  company: Company
                         |  id: ID!
                         |  address: Address
                         |  phone: String
                         |}
                         |""".stripMargin.trim

        for { graphQL <- config.toBlueprint.toGraphQL } yield assertTrue(graphQL.render == expected)
      },
      suite("execute")(
        test("users name") {
          val program = execute(JsonPlaceholderConfig.config)(""" query { users { name } } """)
          assertZIO(program)(equalTo("""{}"""))
        },
        test("posts body") {
          val program = execute(JsonPlaceholderConfig.config)(""" query { posts { body } } """)
          assertZIO(program)(equalTo("""{}"""))
        },
        test("users company") {
          val program = execute(JsonPlaceholderConfig.config)(""" query {  } """)
          assertZIO(program)(equalTo("""{}"""))
        }
      )
    ).provide(GraphQLGenerator.live, TypeGenerator.live, StepGenerator.live, EvaluationRuntime.live)
}
