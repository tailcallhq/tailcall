package tailcall.gateway

import tailcall.gateway.ast.Document
import tailcall.gateway.ast.Document._
import tailcall.gateway.dsl.scala.Orc
import tailcall.gateway.dsl.scala.Orc.Field
import tailcall.gateway.remote._
import tailcall.gateway.service._
import zio.ZIO
import zio.schema.DynamicValue
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
        val document = Document(List(
          SchemaDefinition(query = Some("Query")),
          ObjectTypeDefinition("Query", List(FieldDefinition("foo", Nil, NamedType("Foo", nonNull = false)))),
          ObjectTypeDefinition("Foo", List(FieldDefinition("bar", Nil, NamedType("Bar", nonNull = false)))),
          ObjectTypeDefinition(
            "Bar",
            List(
              FieldDefinition("value", Nil, NamedType("Int", nonNull = false), Option(_ => Remote(DynamicValue(100))))
            )
          )
        ))

        val program = execute(document)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("{\"foo\":{\"bar\":{\"value\":100}}}"))
      },
      test("with nesting array") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {value: Int}
        val document = Document(List(
          SchemaDefinition(query = Some("Query")),
          ObjectTypeDefinition("Query", List(FieldDefinition("foo", Nil, NamedType("Foo", nonNull = false)))),
          ObjectTypeDefinition(
            "Foo",
            List(FieldDefinition(
              "bar",
              Nil,
              ListType(NamedType("Bar", nonNull = false), nonNull = false),
              Option(_ => Remote(DynamicValue(List(100, 200, 300))))
            ))
          ),
          ObjectTypeDefinition(
            "Bar",
            List(
              FieldDefinition("value", Nil, NamedType("Int", nonNull = false), Option(_ => Remote(DynamicValue(100))))
            )
          )
        ))

        val program = execute(document)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("""{"foo":{"bar":[{"value":100},{"value":100},{"value":100}]}}"""))
      },
      test("with nesting array ctx") {
        // type Query {foo: Foo}
        // type Foo {bar: [Bar]}
        // type Bar {value: Int}
        val document = Document(List(
          SchemaDefinition(query = Some("Query")),
          ObjectTypeDefinition("Query", List(FieldDefinition("foo", Nil, NamedType("Foo", nonNull = false)))),
          ObjectTypeDefinition(
            "Foo",
            List(FieldDefinition(
              "bar",
              Nil,
              ListType(NamedType("Bar", nonNull = false), nonNull = false),
              Option(_ => Remote(DynamicValue(List(100, 200, 300))))
            ))
          ),
          ObjectTypeDefinition(
            "Bar",
            List(FieldDefinition(
              "value",
              Nil,
              NamedType("Int", nonNull = false),
              Option(ctx => ctx.path("value").flatMap(_.toTyped[Int]).map(_ + Remote(1)).toDynamic)
            ))
          )
        ))

        val program = execute(document)("query {foo { bar { value }}}")
        assertZIO(program)(equalTo("""{"foo":{"bar":[{"value":101},{"value":201},{"value":301}]}}"""))
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
