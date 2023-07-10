package tailcall.runtime

import caliban.InputValue
import tailcall.runtime.internal.{JSONPlaceholderClient, TValid}
import tailcall.runtime.lambda.Syntax._
import tailcall.runtime.model.Config.{Field, Type}
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.{Config, Context, Path}
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.http.model.Headers
import zio.http.{Request, URL => ZURL}
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}
import zio.test.Assertion.{contains, equalTo}
import zio.test.TestAspect.before
import zio.test.{TestSystem, assertTrue, assertZIO}
import zio.{Chunk, Ref, UIO, ZIO}

import java.net.URI

/**
 * Tests for the generation of GraphQL steps from a config.
 * This is done by writing a test config, converting to
 * graphql and testing it with sample graphql queries.
 */
object ConfigExecutionSpec extends TailcallSpec {
  override def spec =
    suite("Config to GraphQL Step")(
      test("dictionary") {
        val value: Json = Json
          .Obj("a" -> Json.Num(1), "b" -> Json.Obj("k1" -> Json.Num(1), "k2" -> Json.Num(2), "k3" -> Json.Num(3)))

        val transformation = JsonT.applySpec("a" -> JsonT.path("a"), "b" -> JsonT.path("b").pipe(JsonT.toKeyValue))

        val config = Config.default.withTypes(
          "Query" -> Type(
            "z" -> Field.ofType("A").withSteps(Operation.constant(value), Operation.transform(transformation))
          ),
          "A"     -> Type("a" -> Field.int, "b" -> Field.ofType("B").asList),
          "B"     -> Type("key" -> Field.str, "value" -> Field.int),
        )

        val program = resolve(config)("""{z {a b {key value}}}""")
        assertZIO(program)(equalTo(
          """{"z":{"a":1,"b":[{"key":"k1","value":1},{"key":"k2","value":2},{"key":"k3","value":3}]}}"""
        ))
      },
      test("mutation with input type") {
        val config = Config.default.withMutation("Mutation").withTypes(
          "Query"    -> Config.Type("foo" -> Config.Field.ofType("Foo")),
          "Mutation" -> Config.Type(
            "createFoo" -> Config.Field.ofType("Foo").withArguments("input" -> Config.Arg.ofType("FooInput"))
              .resolveWith(Map("a" -> 1))
          ),
          "Foo"      -> Config.Type("a" -> Config.Field.ofType("Int")),
          "FooInput" -> Config.Type(
            "a" -> Config.Field.ofType("Int"),
            "b" -> Config.Field.ofType("Int"),
            "c" -> Config.Field.ofType("Int"),
          ),
        )

        val program = resolve(config, Map.empty)("mutation {createFoo(input: {a: 1}){a}}")
        assertZIO(program)(equalTo("""{"createFoo":{"a":1}}"""))
      },
      suite("modified")(
        test("resolve using env variables") {
          val config = Config.default.withTypes(
            "Query" -> Config.Type("identity" -> Config.Field.int.resolveWithFunction(_.path("env", "foo").toDynamic))
          )
          for {
            json <- resolve(config, Map.empty)("""query {identity}""")
          } yield assertTrue(json == """{"identity":"bar"}""")
        },
        test("resolve with headers") {
          val config = Config.default.withTypes(
            "Query" -> Config
              .Type("identity" -> Config.Field.str.resolveWithFunction(_.path("headers", "authorization").toDynamic))
          )
          for {
            json <- resolve(config, Map.empty)("""query {identity}""")
          } yield assertTrue(json == """{"identity":"bar"}""")
        },
      ),
      test("with no base url") {
        val http   = Operation.Http(Path.unsafe.fromString("/users"))
        val config = Config.default.withTypes("Query" -> Type("foo" -> Config.Field.int.withSteps(http)))
        val errors = config.toBlueprint

        assertTrue(
          errors == TValid.fail("No base URL defined in the server configuration").trace("Query", "foo", "@unsafe")
        )
      },
      test("with local url") {
        val http   = Operation.Http(Path.unsafe.fromString("/users/1"))
          .withBaseURL(URI.create("https://jsonplaceholder.typicode.com").toURL)
        val config = Config.default.withTypes(
          "Query" -> Type("users" -> Config.Field.ofType("User").withSteps(http)),
          "User"  -> Type(
            "id"    -> Config.Field.ofType("Int"),
            "name"  -> Config.Field.ofType("String"),
            "email" -> Config.Field.ofType("String"),
          ),
        )

        for {
          json <- resolve(config, Map.empty)("""query {users {name}}""")
        } yield assertTrue(json == """{"users":{"name":"Leanne Graham"}}""")
      },
      suite("unsafe")(test("with http") {
        val http   = Operation.Http(Path.unsafe.fromString("/users"))
        val config = Config.default.withBaseURL(URI.create("https://jsonplaceholder.typicode.com").toURL)
          .withTypes("Query" -> Type("foo" -> Config.Field.ofType("Foo").withSteps(http).withHttp(http)))

        val errors = config.toBlueprint

        assertTrue(errors == TValid.fail("can not be used with @unsafe").trace("Query", "foo", "@http"))
      }),
      suite("inline")(test("http directive") {
        val config = Config.default.withBaseURL(URI.create("https://jsonplaceholder.typicode.com").toURL).withTypes(
          "Query" -> Config
            .Type("user" -> Config.Field.ofType("User").withHttp(Operation.Http(Path.unsafe.fromString("/users/1")))),
          "User"  -> Config.Type("id" -> Config.Field.ofType("Int"), "name" -> Config.Field.ofType("String")),
        )

        for {
          json <- resolve(config, Map.empty)("""query {user {id name}}""")
        } yield assertTrue(json == """{"user":{"id":1,"name":"Leanne Graham"}}""")
      }),
      suite("context")(
        test("one level") {
          val program = collect { ref =>
            Config.default
              .withTypes("Query" -> Config.Type("a" -> Config.Field.int.resolveWithFunction(_.tap(ref.insert(_)))))
          }

          val expected = context(())
          assertZIO(program("query {a}"))(contains(expected))
        },
        test("two levels") {
          val program = collect { ref =>
            Config.default.withTypes(
              "Query" -> Config.Type("a" -> Config.Field.ofType("A").resolveWith(100)),
              "A"     -> Config.Type("b" -> Config.Field.int.resolveWithFunction(_.tap(ref.insert(_)))),
            )
          }

          val expected = context(value = 100, parent = Option(context(value = 100)))
          assertZIO(program("query {a {b}}"))(contains(expected))
        },
        test("three levels") {
          val program = collect { ref =>
            Config.default.withTypes(
              "Query" -> Config.Type("a" -> Config.Field.ofType("A").resolveWith(100)),
              "A"     -> Config.Type("b" -> Config.Field.ofType("B").resolveWith(200)),
              "B"     -> Config.Type("c" -> Config.Field.int.resolveWithFunction(_.tap(ref.insert(_)))),
            )
          }

          val expected =
            context(value = 200, parent = Option(context(value = 200, parent = Option(context(value = 100)))))
          assertZIO(program("query {a {b {c}}}"))(contains(expected))
        },
        test("four levels") {
          val program = collect { ref =>
            Config.default.withTypes(
              "Query" -> Config.Type("a" -> Config.Field.ofType("A").resolveWith(100)),
              "A"     -> Config.Type("b" -> Config.Field.ofType("B").resolveWith(200)),
              "B"     -> Config.Type("c" -> Config.Field.ofType("C").resolveWith(300)),
              "C"     -> Config.Type("d" -> Config.Field.int.resolveWithFunction(_.tap(ref.insert(_)))),
            )
          }

          val expected = context(
            value = 300,
            parent =
              Option(context(value = 300, parent = Option(context(value = 200, parent = Option(context(value = 100)))))),
          )
          assertZIO(program("query {a {b {c {d}}}}"))(contains(expected))
        },
        test("one level with list") {
          val program = collect { ref =>
            Config.default.withTypes(
              "Query" -> Config.Type("a" -> Config.Field.ofType("A").asList.resolveWith(List(100, 200))),
              "A"     -> Config
                .Type("b" -> Config.Field.int.resolveWithFunction(_.tap(ref.insert(_)).path("value").toDynamic)),
            )
          }

          val expected = context(value = 200, parent = Option(context(value = Chunk(100, 200))))
          assertZIO(program("query {a{b}}"))(contains(expected))
        },
        test("two level with list") {
          val program = collect { ref =>
            Config.default.withTypes(
              "Query" -> Config.Type("a" -> Config.Field.ofType("A").asList.resolveWith(List(100, 101))),
              "A"     -> Config.Type("b" -> Config.Field.ofType("B").asList.resolveWith(List(200, 201))),
              "B"     -> Config
                .Type("c" -> Config.Field.int.resolveWithFunction(_.tap(ref.insert(_)).path("value").toDynamic)),
            )
          }

          val expected = context(
            value = 201,
            parent = Option(context(value = Chunk(200, 201), parent = Option(context(value = Chunk(100, 101))))),
          )
          assertZIO(program("query {a{b{c}}}"))(contains(expected))
        },
        test("three level with list") {
          val program = collect { ref =>
            Config.default.withTypes(
              "Query" -> Config.Type("a" -> Config.Field.ofType("A").asList.resolveWith(List(100, 101))),
              "A"     -> Config.Type("b" -> Config.Field.ofType("B").asList.resolveWith(List(200, 201))),
              "B"     -> Config.Type("c" -> Config.Field.ofType("C").asList.resolveWith(List(300, 301))),
              "C"     -> Config.Type("d" -> Config.Field.int.resolveWithFunction(_.tap(ref.insert(_)).toDynamic)),
            )
          }

          val expected = context(
            value = 301,
            parent = Option(context(
              value = Chunk(300, 301),
              parent = Option(context(value = Chunk(200, 201), parent = Option(context(value = Chunk(100, 101))))),
            )),
          )
          assertZIO(program("query {a{b{c{d}}}}"))(contains(expected))
        },
      ),
    ).provide(
      GraphQLGenerator.default,
      JSONPlaceholderClient.default,
      HttpContext.live(Some(Request.get(ZURL.empty).addHeaders(Headers("authorization", "bar")))),
    ) @@ before(TestSystem.putEnv("foo", "bar"))

  private def collect(
    f: RefList[DynamicValue] => Config
  ): String => ZIO[HttpContext with GraphQLGenerator, Throwable, List[Context]] = { q =>
    for {
      ref <- Ref.make[List[DynamicValue]](List.empty).map(RefList(_))
      config = f(ref)
      _       <- resolve(config)(q)
      data    <- ref.get
      context <- ZIO.foreach(data)(data =>
        ZIO.fromEither(data.toTypedValue[Context]) <> ZIO.fail(new Exception("Could not convert to context"))
      )
    } yield context
  }

  private def context[A: Schema](
    value: A,
    args: Map[String, DynamicValue] = Map.empty,
    parent: Option[Context] = None,
  ): Context =
    Context(
      Schema[A].toDynamic(value),
      env = Map("foo" -> "bar"),
      headers = Map("authorization" -> "bar"),
      args = args,
      parent = parent,
    )

  private def resolve(config: Config, variables: Map[String, InputValue] = Map.empty)(
    query: String
  ): ZIO[HttpContext with GraphQLGenerator, Throwable, String] = {
    for {
      blueprint   <- Transcoder.toBlueprint(config).toTask
      graphQL     <- blueprint.toGraphQL
      interpreter <- graphQL.interpreter
      result      <- interpreter.execute(query, variables = variables)
      _           <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
  }

  final private case class RefList[A](ref: Ref[List[A]]) {
    def get: UIO[List[A]] = ref.get

    def insert(value: A): UIO[Unit] = ref.update(_ :+ value)
  }
}
