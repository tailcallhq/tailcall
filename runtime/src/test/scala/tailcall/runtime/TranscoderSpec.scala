package tailcall.runtime

import tailcall.runtime.ast.{Endpoint, TSchema}
import tailcall.runtime.http.Method
import tailcall.runtime.transcoder.Transcoder
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, TestResult, ZIOSpecDefault, assertZIO}
import zio.{Scope, ZIO}

object TranscoderSpec extends ZIOSpecDefault {
  private val User = TSchema
    .obj("username" -> TSchema.String, "id" -> TSchema.Int, "name" -> TSchema.String, "email" -> TSchema.String)

  private val InputUser = TSchema.obj("username" -> TSchema.String, "name" -> TSchema.String, "email" -> TSchema.String)

  private val jsonEndpoint = Endpoint.make("jsonplaceholder.typicode.com").withHttps

  def assertSchema(endpoint: Endpoint)(expected: String): ZIO[Any, String, TestResult] = {
    val schema = Transcoder.toConfig(endpoint).flatMap(Transcoder.toGraphQLSchema).map(_.stripMargin.trim)
    assertZIO(schema.toZIO)(equalTo(expected.trim))
  }

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("TranscoderSpec")(
      suite("json to TSchema")(
        test("object to tSchema") {
          val json    = """{"a": 1, "b": "2", "c": true, "d": null, "e": [1, 2, 3]}"""
          val tSchema = TSchema.obj(
            "a" -> TSchema.Int,
            "b" -> TSchema.String,
            "c" -> TSchema.Boolean,
            "d" -> TSchema.empty,
            "e" -> TSchema.arr(TSchema.Int),
          )
          assertZIO(Transcoder.toTSchema(json).toZIO)(equalTo(tSchema))
        },
        test("array tSchema") {
          val json    = """[{"a": 1, "b": true}, {"a": 1, "b": true}, {"a": 2, "b": false}]"""
          val tSchema = TSchema.arr(TSchema.obj("a" -> TSchema.Int, "b" -> TSchema.Boolean))
          assertZIO(Transcoder.toTSchema(json).toZIO)(equalTo(tSchema))
        },
        test("nullables to TSchema") {
          val json    = """[{"a": 1}, {"a": null}]"""
          val tSchema = TSchema.arr(TSchema.obj("a" -> (TSchema.Int.opt)))
          assertZIO(Transcoder.toTSchema(json).toZIO)(equalTo(tSchema))
        },
      ),
      suite("endpoint to config")(
        test("output schema") {
          val endpoint = jsonEndpoint.withHttps.withOutput(Option(User.arr)).withPath("/users")
          val expected = """
                           |schema @server(baseURL: "https://jsonplaceholder.typicode.com") {
                           |  query: Query
                           |}
                           |
                           |type Query {
                           |  field: [Type!] @steps(value: [{http: {path: "/users"}}])
                           |}
                           |
                           |type Type {
                           |  username: String!
                           |  id: Int!
                           |  name: String!
                           |  email: String!
                           |}
                           |""".stripMargin
          assertSchema(endpoint)(expected.trim)
        },
        test("argument schema") {
          val endpoint = jsonEndpoint.withOutput(Option(User.opt))
            .withInput(Option(TSchema.obj("userId" -> TSchema.Int))).withPath("/user")

          val expected = """
                           |schema @server(baseURL: "https://jsonplaceholder.typicode.com") {
                           |  query: Query
                           |}
                           |
                           |type Query {
                           |  field(userId: Int!): Type @steps(value: [{http: {path: "/user"}}])
                           |}
                           |
                           |type Type {
                           |  username: String!
                           |  id: Int!
                           |  name: String!
                           |  email: String!
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
              |type Mutation {
              |  field(username: String!, name: String!, email: String!): Type! @steps(value: [{http: {path: "/user",method: "POST"}}])
              |}
              |
              |type Query
              |
              |type Type {
              |  username: String!
              |  id: Int!
              |  name: String!
              |  email: String!
              |}
              |""".stripMargin
          assertSchema(endpoint)(expected.trim)
        },
      ),
    )
}
