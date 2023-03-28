package tailcall.runtime

import tailcall.runtime.ast.{Endpoint, TSchema}
import tailcall.runtime.http.Method
import tailcall.runtime.transcoder.{Endpoint2Config, Transcoder}
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, TestResult, ZIOSpecDefault, assertZIO}
import zio.{Scope, ZIO}

object Endpoint2ConfigSpec extends ZIOSpecDefault with Endpoint2Config {
  private val User = TSchema
    .obj("username" -> TSchema.String, "id" -> TSchema.Int, "name" -> TSchema.String, "email" -> TSchema.String)

  private val InputUser = TSchema.obj("username" -> TSchema.String, "name" -> TSchema.String, "email" -> TSchema.String)

  private val jsonEndpoint = Endpoint.make("jsonplaceholder.typicode.com").withHttps

  def assertSchema(endpoint: Endpoint)(expected: String): ZIO[Any, String, TestResult] = {
    val schema = toConfig(endpoint).flatMap(Transcoder.toGraphQLSchema).map(_.stripMargin.trim)
    assertZIO(schema.toZIO)(equalTo(expected.trim))
  }

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("TranscoderSpec")(suite("endpoint to config")(
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
        val endpoint = jsonEndpoint.withOutput(Option(User.opt)).withInput(Option(TSchema.obj("userId" -> TSchema.Int)))
          .withPath("/user")

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
    ))
}
