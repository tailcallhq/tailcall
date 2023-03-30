package tailcall.runtime

import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.service._
import zio.durationInt
import zio.test.TestAspect.timeout
import zio.test.{ZIOSpecDefault, assertTrue}

object Config2GraphQLSchemaSpec extends ZIOSpecDefault {
  override def spec =
    suite("config to graphql schema")(test("schema") {
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
                        |type Id {
                        |  id: Int!
                        |}
                        |
                        |type Mutation {
                        |  createUser(user: NewUser!): Id
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
                        |  posts: [Post]
                        |  users: [User]
                        |  post(id: Int!): Post
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

      config.toBlueprint.toGraphQL.map(graphQL => assertTrue(graphQL.render == expected))
    }).provide(GraphQLGenerator.default) @@ timeout(10 seconds)
}
