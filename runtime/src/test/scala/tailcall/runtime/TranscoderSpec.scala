package tailcall.runtime

import tailcall.runtime.ast.{Endpoint, TSchema}
import tailcall.runtime.transcoder.Transcoder
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertZIO}

object TranscoderSpec extends ZIOSpecDefault {
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
        //
        test("endpoint to config") {
          val User = TSchema
            .obj("username" -> TSchema.String, "id" -> TSchema.Int, "name" -> TSchema.String, "email" -> TSchema.String)

          val endpoint = Endpoint.make("jsonplaceholder.typicode.com").withHttps.withOutput(Option(User.arr))
            .withPath("/users")

          val schema = Transcoder.toConfig(endpoint).flatMap(Transcoder.toGraphQLSchema).map(_.stripMargin.trim)

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
          assertZIO(schema.toZIO)(equalTo(expected.trim))
        }
      ),
    )
}
