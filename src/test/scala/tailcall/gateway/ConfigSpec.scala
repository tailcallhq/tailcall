package tailcall.gateway

import tailcall.gateway.internal.{Extension, JsonPlaceholderConfig}
import tailcall.gateway.service.{EvaluationRuntime, GraphQLGenerator, StepGenerator, TypeGenerator}
import zio.test.{ZIOSpecDefault, assertTrue}

object ConfigSpec extends ZIOSpecDefault {
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
      test("render") {
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
      }
    ).provide(GraphQLGenerator.live, TypeGenerator.live, StepGenerator.live, EvaluationRuntime.live)
}
