package tailcall.runtime

import tailcall.runtime.internal.JsonPlaceholderConfig
import tailcall.runtime.model.Config
import tailcall.runtime.model.Config.{Arg, Field, Type}
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Transcoder
import zio.durationInt
import zio.test.TestAspect.timeout
import zio.test.{ZIOSpecDefault, assertTrue}

object Config2GraphQLSchemaSpec extends ZIOSpecDefault {
  override def spec =
    suite("config to graphql schema")(
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
        config.toBlueprint.toGraphQL.map(graphQL => assertTrue(graphQL.render == expected))
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
        config.toBlueprint.toGraphQL.map(graphQL => assertTrue(graphQL.render == expected))
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

        Transcoder.toGraphQLSchema(config).toZIO.map(schema => assertTrue(schema == expected))
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

        Transcoder.toGraphQLSchema(config).toZIO.map(schema => assertTrue(schema == expected))
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

        Transcoder.toGraphQLSchema(config).toZIO.map(schema => assertTrue(schema == expected))
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
        config.toBlueprint.toGraphQL.map(graphQL => assertTrue(graphQL.render == expected))

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
          Transcoder.toGraphQLSchema(config).toZIO.map(schema => assertTrue(schema == expected))
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
          Transcoder.toGraphQLSchema(config).toZIO.map(schema => assertTrue(schema == expected))
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
                            |  baz: String
                            |}
                            |
                            |type Query {
                            |  foo(input: Foo): Int
                            |}
                            |""".stripMargin.trim
          Transcoder.toGraphQLSchema(config).toZIO.map(schema => assertTrue(schema == expected))
        },
      ),
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
                          |  zip: String
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
                          |  "A list of all posts."
                          |  posts: [Post]
                          |  "A list of all users."
                          |  users: [User]
                          |  "A single post by id."
                          |  post(id: Int!): Post
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

        Transcoder.toGraphQLSchema(config).toZIO.map(schema => assertTrue(schema == expected))
      },
    ).provide(GraphQLGenerator.default) @@ timeout(10 seconds)
}
