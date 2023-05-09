package tailcall.runtime.transcoder

import tailcall.runtime.http.Scheme
import tailcall.runtime.model.Config.Field
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model._
import zio.Chunk
import zio.test.Assertion.equalTo
import zio.test._

import java.net.URI

object Config2BlueprintSpec extends ZIOSpecDefault {
  def spec =
    suite("Config to Blueprint")(
      test("timeout") {
        val config  =
          Config(server = Server(baseURL = Some(URI.create("http://localhost:8080").toURL), timeout = Some(1000)))
        val timeout = Transcoder.toBlueprint(config).toOption.flatMap(_.server.globalResponseTimeout)

        assertTrue(timeout == Option(1000))
      },
      test("cyclic types") {
        val config = Config.default.withBaseURL(URI.create("https://jsonplaceholder.com").toURL).withTypes(
          "Query" -> Config.Type("users" -> Config.Field.ofType("User").asList),
          "User"  -> Config.Type(
            "name"  -> Config.Field.string,
            "id"    -> Config.Field.int,
            "posts" -> Config.Field.ofType("Post").asList.withHttp(Http(path = Path.unsafe.fromString("/posts"))),
          ),
          "Post"  -> Config.Type(
            "name" -> Config.Field.string,
            "id"   -> Config.Field.int,
            "user" -> Config.Field.ofType("User").withHttp(Http(path = Path.unsafe.fromString("/users"))),
          ),
        )

        assertTrue(Transcoder.toBlueprint(config).nonEmpty)
      },
      suite("required")(
        test("http with required") {
          val config = Config.default
            .withTypes("Query" -> Config.Type("foo" -> Config.Field.string.asRequired.withHttp(Http(Path.empty))))
          assertZIO(Transcoder.toBlueprint(config).toZIO.flip)(equalTo(
            Chunk("""`Query.foo` has an http operation hence can not be non-nullable""")
          ))
        },
        test("unsafe with required") {
          val config = Config.default
            .withTypes("Query" -> Config.Type("foo" -> Config.Field.string.asRequired.resolveWith(100)))
          assertZIO(Transcoder.toBlueprint(config).toZIO.flip)(equalTo(
            Chunk("""`Query.foo` has an unsafe operation hence can not be non-nullable""")
          ))
        },
      ),
      test("endpoint") {
        val config    = Config.default.withBaseURL("https://foo.com")
          .withTypes("Query" -> Config.Type("foo" -> Config.Field.string.withHttp(Http.fromPath("/users"))))
        val endpoints = Transcoder.toBlueprint(config).map(_.endpoints).toZIO
        val expected  =
          List(Endpoint.make("foo.com").withScheme(Scheme.Https).withPath("/users").withOutput(Option(TSchema.str.opt)))
        assertZIO(endpoints)(equalTo(expected))
      },
      test("endpoint output schema") {
        val config  = Config.default.withBaseURL("http://abc.com").withTypes(
          "Query" -> Config.Type("foo" -> Field.ofType("Foo")),
          "Foo"   -> Config.Type(
            "a" -> Field.ofType("Foo").withHttp(Http.fromPath("/a")),
            "b" -> Field.ofType("String"),
            "c" -> Field.ofType("String"),
          ),
        )
        val schemas = Transcoder.toBlueprint(config).map(_.endpoints.flatMap(_.output.toList)).toTask.orDie

        val expected = List(TSchema.obj("b" -> TSchema.str.opt, "c" -> TSchema.str.opt).opt)
        assertZIO(schemas)(equalTo(expected))
      },
    )
}
