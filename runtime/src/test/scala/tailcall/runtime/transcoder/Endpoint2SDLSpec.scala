package tailcall.runtime.transcoder

import tailcall.TailcallSpec
import tailcall.runtime.internal.TValid.Cause
import tailcall.runtime.model.{Endpoint, Method, TSchema}
import tailcall.runtime.transcoder.Endpoint2Config.NameGenerator
import zio.test.{Spec, TestEnvironment, TestResult, assertTrue}
import zio.{Chunk, Scope, ZIO}

object Endpoint2SDLSpec extends TailcallSpec {
  private val User = TSchema
    .obj("username" -> TSchema.Str, "id" -> TSchema.Num, "name" -> TSchema.Str, "email" -> TSchema.Str)

  private val InputUser = TSchema.obj("username" -> TSchema.Str, "name" -> TSchema.Str, "email" -> TSchema.Str)

  private val jsonEndpoint = Endpoint.make("jsonplaceholder.typicode.com").withHttps

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("endpoint to graphql schema")(
      test("output schema") {
        val endpoint = jsonEndpoint.withHttps.withOutput(Option(User.arr)).withPath("/users")
        val expected = """
                         |schema @server(baseURL: "https://jsonplaceholder.typicode.com") {
                         |  query: Query
                         |}
                         |
                         |type Query {
                         |  fieldType_1: [Type_1!] @unsafe(steps: [{http: {path: "/users"}}])
                         |}
                         |
                         |type Type_1 {
                         |  email: String!
                         |  id: Int!
                         |  name: String!
                         |  username: String!
                         |}
                         |""".stripMargin
        assertSchema(endpoint)(expected.trim)
      },
      test("nested output schema") {
        val output   = TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("c" -> TSchema.num)))
        val endpoint = Endpoint.make("abc.com").withOutput(Option(output)).withPath("/abc")
        val expected = """
                         |schema @server(baseURL: "http://abc.com") {
                         |  query: Query
                         |}
                         |
                         |type Query {
                         |  fieldType_1: Type_1! @unsafe(steps: [{http: {path: "/abc"}}])
                         |}
                         |
                         |type Type_1 {
                         |  a: Type_2!
                         |}
                         |
                         |type Type_2 {
                         |  b: Type_3!
                         |}
                         |
                         |type Type_3 {
                         |  c: Int!
                         |}
                         |
                         |""".stripMargin
        assertSchema(endpoint)(expected.trim)
      },
      test("argument schema") {
        val endpoint = jsonEndpoint.withOutput(Option(User.opt)).withInput(Option(TSchema.obj("userId" -> TSchema.Num)))
          .withPath("/user")

        val expected = """
                         |schema @server(baseURL: "https://jsonplaceholder.typicode.com") {
                         |  query: Query
                         |}
                         |
                         |input Type_2 {
                         |  userId: Int!
                         |}
                         |
                         |type Query {
                         |  fieldType_1(value: Type_2!): Type_1 @unsafe(steps: [{http: {path: "/user"}}])
                         |}
                         |
                         |type Type_1 {
                         |  email: String!
                         |  id: Int!
                         |  name: String!
                         |  username: String!
                         |}
                         |""".stripMargin
        assertSchema(endpoint)(expected.trim)
      },
      test("nested argument schema") {
        val endpoint = Endpoint.make("abc.com")
          .withInput(Option(TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("c" -> TSchema.num)))))
          .withOutput(Option(TSchema.num))

        val expected = """
                         |schema @server(baseURL: "http://abc.com") {
                         |  query: Query
                         |}
                         |
                         |input Type_1 {
                         |  a: Type_2!
                         |}
                         |
                         |input Type_2 {
                         |  b: Type_3!
                         |}
                         |
                         |input Type_3 {
                         |  c: Int!
                         |}
                         |
                         |type Query {
                         |  fieldInt(value: Type_1!): Int! @unsafe(steps: [{http: {path: ""}}])
                         |}
                         |""".stripMargin
        assertSchema(endpoint)(expected.trim)
      },
      test("mutation schema") {
        val endpoint = jsonEndpoint.withOutput(Option(User)).withInput(Option(InputUser)).withPath("/user")
          .withMethod(Method.POST)

        val expected =
          """
            |schema @server(baseURL: "https://jsonplaceholder.typicode.com") {
            |  query: Query
            |  mutation: Mutation
            |}
            |
            |input Type_2 {
            |  email: String!
            |  name: String!
            |  username: String!
            |}
            |
            |type Mutation {
            |  fieldType_1(value: Type_2!): Type_1! @unsafe(steps: [{http: {path: "/user",method: "POST"}}])
            |}
            |
            |type Query
            |
            |type Type_1 {
            |  email: String!
            |  id: Int!
            |  name: String!
            |  username: String!
            |}
            |""".stripMargin
        assertSchema(endpoint)(expected.trim)
      },
    )

  private def assertSchema(endpoint: Endpoint)(expected: String): ZIO[Any, Chunk[Cause[String]], TestResult] = {
    val schema = Transcoder.toSDL(endpoint, NameGenerator.incremental).map(_.stripMargin.trim)
    for { result <- schema.toZIO } yield assertTrue(result == expected)
  }
}
