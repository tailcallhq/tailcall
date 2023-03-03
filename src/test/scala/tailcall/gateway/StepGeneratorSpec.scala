package tailcall.gateway

import tailcall.gateway.ast.Document
import tailcall.gateway.dsl.scala.Orc
import tailcall.gateway.dsl.scala.Orc.Field
import tailcall.gateway.remote._
import tailcall.gateway.service._
import zio.ZIO
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

object StepGeneratorSpec extends ZIOSpecDefault {

  def spec = {
    suite("StepGenerator")(
      test("static value") {
        val orc     = Orc("Query" -> List("id" -> Field.output.as("String").resolveWith(100)))
        val program = execute(orc)("query {id}")
        assertZIO(program)(equalTo("""{"id":100}"""))
      },
      test("with args") {
        val orc     = Orc(
          "Query" -> List(
            "sum" -> Field.output.as("Int").withArgument("a" -> Field.input.as("Int"), "b" -> Field.input.as("Int"))
              .withResolver { ctx =>
                {
                  (for {
                    anyA <- ctx.path("args", "a")
                    anyB <- ctx.path("args", "b")
                    a    <- anyA.toTyped[Int]
                    b    <- anyB.toTyped[Int]
                  } yield a + b).toDynamic
                }
              }
          )
        )
        val program = execute(orc)("query {sum(a: 1, b: 2)}")
        assertZIO(program)(equalTo("""{"sum":3}"""))
      },
      test("with nesting") {
        // type Query {foo: Foo}
        // type Foo {bar: Bar}
        // type Bar {value: Int}

        val orc = Orc(
          "Query" -> List("foo" -> Field.output.as("Foo")),
          "Foo"   -> List("bar" -> Field.output.as("Bar")),
          "Bar"   -> List("value" -> Field.output.as("Int").resolveWith(100))
        )

        val program = execute(orc)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("{\"foo\":{\"bar\":{\"value\":100}}}"))
      },
      test("with nesting array") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {value: Int}

        val orc = Orc(
          "Query" -> List("foo" -> Field.output.as("Foo")),
          "Foo"   -> List("bar" -> Field.output.asList("Bar").resolveWith(List(100, 200, 300))),
          "Bar"   -> List("value" -> Field.output.as("Int").resolveWith(100))
        )

        val program = execute(orc)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("""{"foo":{"bar":[{"value":100},{"value":100},{"value":100}]}}"""))
      },
      test("with nesting array ctx") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {value: Int}
        val orc = Orc(
          "Query" -> List("foo" -> Field.output.as("Foo")),
          "Foo"   -> List("bar" -> Field.output.asList("Bar").resolveWith(List(100, 200, 300))),
          "Bar"   -> List("value" -> Field.output.as("Int").withResolver {
            _.path("value").flatMap(_.toTyped[Int].map(_ + Remote(1))).toDynamic
          })
        )

        val program = execute(orc)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("""{"foo":{"bar":[{"value":101},{"value":201},{"value":301}]}}"""))
      },
      test("with nesting level 3") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {baz: [Baz]}
        // type Baz{value: Int}
        val orc = Orc(
          "Query" -> List("foo" -> Field.output.as("Foo")),
          "Foo"   -> List("bar" -> Field.output.asList("Bar").resolveWith(List(100, 200, 300))),
          "Bar"   -> List(
            "baz" -> Field.output.asList("Baz")
              .withResolver(_.path("value").flatMap(_.toTyped[Int].map(_ + Remote(1))).toDynamic)
          ),
          "Baz"   -> List("value" -> Field.output.as("Int").withResolver {
            _.path("value").flatMap(_.toTyped[Option[Int]]).flatMap(identity(_)).map(_ + Remote(1)).toDynamic
          })
        )

        val program = execute(orc)("query {foo { bar { baz {value} }}}")
        assertZIO(program)(equalTo(
          """{"foo":{"bar":[{"baz":[{"value":102}]},{"baz":[{"value":202}]},{"baz":[{"value":302}]}]}}"""
        ))
      }
    ).provide(GraphQLGenerator.live, TypeGenerator.live, StepGenerator.live, EvaluationRuntime.live)
  }

  def execute(orc: Orc)(query: String): ZIO[GraphQLGenerator, Throwable, String] =
    orc.toDocument.flatMap(execute(_)(query))

  def execute(doc: Document)(query: String): ZIO[GraphQLGenerator, Throwable, String] =
    for {
      graphQL     <- doc.toGraphQL
      interpreter <- graphQL.interpreter
      result <- interpreter.execute(query, skipValidation = true) // TODO: enable validation after __type is available
      _      <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
}
