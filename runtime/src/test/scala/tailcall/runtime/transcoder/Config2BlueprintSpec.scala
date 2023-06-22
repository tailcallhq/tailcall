package tailcall.runtime.transcoder

import tailcall.runtime.http.Scheme
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Config.Field
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model._
import tailcall.test.TailcallSpec
import zio.Chunk
import zio.test.Assertion.equalTo
import zio.test._

import java.net.URI

object Config2BlueprintSpec extends TailcallSpec {
  def spec =
    suite("Config to Blueprint")(
      test("cyclic types") {
        val config = Config.default.withBaseURL(URI.create("https://jsonplaceholder.com").toURL).withTypes(
          "Query" -> Config.Type("users" -> Config.Field.ofType("User").asList),
          "User"  -> Config.Type(
            "name"  -> Config.Field.str,
            "id"    -> Config.Field.int,
            "posts" -> Config.Field.ofType("Post").asList.withHttp(Http(path = Path.unsafe.fromString("/posts"))),
          ),
          "Post"  -> Config.Type(
            "name" -> Config.Field.str,
            "id"   -> Config.Field.int,
            "user" -> Config.Field.ofType("User").withHttp(Http(path = Path.unsafe.fromString("/users"))),
          ),
        )

        assertTrue(Transcoder.toBlueprint(config).isValid)
      },
      suite("required")(
        test("http with required") {
          val config = Config.default
            .withTypes("Query" -> Config.Type("foo" -> Config.Field.str.asRequired.withHttp(Http(Path.empty))))

          val expected =
            Chunk(TValid.Cause("""can not be used with non-nullable fields""", "Query" :: "foo" :: "@http" :: Nil))
          assertZIO(Transcoder.toBlueprint(config).toZIO.flip)(equalTo(expected))
        },
        test("unsafe with required") {
          val config = Config.default
            .withTypes("Query" -> Config.Type("foo" -> Config.Field.str.asRequired.resolveWith(100)))

          val expected =
            Chunk(TValid.Cause("can not be used with non-nullable fields", "Query" :: "foo" :: "@unsafe" :: Nil))

          assertZIO(Transcoder.toBlueprint(config).toZIO.flip)(equalTo(expected))
        },
      ),
      test("endpoint") {
        val config    = Config.default.withBaseURL("https://foo.com")
          .withTypes("Query" -> Config.Type("foo" -> Config.Field.str.withHttp(Http.fromPath("/users"))))
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
      test("extends with duplicate field") {
        val config = Config.default.withBaseURL(URI.create("http://foo.com").toURL).withTypes(
          "Identified" -> Config.Type("id" -> Config.Field.int),
          "User"       -> Config.Type("name" -> Config.Field.str).extendsWith("Identified"),
          "UserQuery"  -> Config.Type(
            "name"  -> Config.Field.str,
            "posts" -> Config.Field.ofType("Post").asList
              .withHttp(Http(path = Path.unsafe.fromString("/users/{{value.id}}/posts"))),
          ).extendsWith("User"),
          "Post"       -> Config.Type("title" -> Config.Field.str),
          "Query"      -> Config
            .Type("users" -> Config.Field.ofType("User").asList.withHttp(Http(path = Path.unsafe.fromString("/users")))),
        )

        val expected = Chunk(TValid.Cause("""Duplicate field found for UserQuery""", Nil))
        assertZIO(Transcoder.toBlueprint(config).toZIO.flip)(equalTo(expected))
      },
      test("extends with missing parent type") {
        val config = Config.default.withBaseURL(URI.create("http://foo.com").toURL).withTypes(
          // "Identified" -> Config.Type("id" -> Config.Field.int),
          "User"      -> Config.Type("name" -> Config.Field.str).extendsWith("Identified"),
          "UserQuery" -> Config.Type(
            "posts" -> Config.Field.ofType("Post").asList
              .withHttp(Http(path = Path.unsafe.fromString("/users/{{value.id}}/posts")))
          ).extendsWith("User"),
          "Post"      -> Config.Type("title" -> Config.Field.str),
          "Query"     -> Config
            .Type("users" -> Config.Field.ofType("User").asList.withHttp(Http(path = Path.unsafe.fromString("/users")))),
        )

        val expected = Chunk(
          TValid.Cause("""Could not find definition for Identified""", Nil),
          TValid.Cause("""Could not find definition for Identified""", Nil),
        )
        assertZIO(Transcoder.toBlueprint(config).toZIO.flip)(equalTo(expected))

      },
      test("extends") {
        val config = Config.default.withBaseURL(URI.create("http://foo.com").toURL).withTypes(
          "Identified" -> Config.Type("id" -> Config.Field.int),
          "User"       -> Config.Type("name" -> Config.Field.str).extendsWith("Identified"),
          "UserQuery"  -> Config.Type(
            "posts" -> Config.Field.ofType("Post").asList
              .withHttp(Http(path = Path.unsafe.fromString("/users/{{value.id}}/posts")))
          ).extendsWith("User"),
          "Post"       -> Config.Type("title" -> Config.Field.str),
          "Query"      -> Config
            .Type("users" -> Config.Field.ofType("User").asList.withHttp(Http(path = Path.unsafe.fromString("/users")))),
        )

        assertTrue(Transcoder.toBlueprint(config).isValid)
      },
    )
}
