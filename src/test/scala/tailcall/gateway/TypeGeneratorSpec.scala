package tailcall.gateway

import tailcall.gateway.ast.Blueprint
import tailcall.gateway.dsl.scala.Orc.Field
import tailcall.gateway.dsl.scala.Orc.Type.{ListType, NamedType, NonNull}
import tailcall.gateway.dsl.scala.{Orc, OrcBlueprint}
import tailcall.gateway.service._
import zio.ZIO
import zio.test.Assertion._
import zio.test._

object TypeGeneratorSpec extends ZIOSpecDefault {
  override def spec =
    suite("DocumentTypeGenerator")(
      test("document type generation") {
        val orc = Orc("Query" -> List("test" -> Field.output.to("String").resolveWith("test")))

        val actual   = render(orc)
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test: String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      },
      test("document with InputValue") {
        val orc    = Orc(
          "Query" -> List(
            "test" -> Field.output.to("String").resolveWith("test")
              .withArgument("arg" -> Field.input.to("String").withDefault("test"))
          )
        )
        val actual = render(orc)

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      },
      test("blueprint with InputValue and default") {
        val orc    = Orc(
          "Query" -> List(
            "test" -> Field.output.to("String").resolveWith("test")
              .withArgument("arg" -> Field.input.to("String").withDefault("test"))
          )
        )
        val actual = render(orc)

        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Query {
                          |  test(arg: String = "test"): String
                          |}""".stripMargin
        assertZIO(actual)(equalTo(expected))
      },
      test("with nesting") {
        val orc      = Orc(
          "Query" -> List("foo" -> Field.output.to("Foo")),
          "Foo"   -> List("bar" -> Field.output.to("Bar")),
          "Bar"   -> List("value" -> Field.output.to("Int").resolveWith(100))
        )
        val schema   = render(orc)
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Bar {
                          |  value: Int
                          |}
                          |
                          |type Foo {
                          |  bar: Bar
                          |}
                          |
                          |type Query {
                          |  foo: Foo
                          |}""".stripMargin
        assertZIO(schema)(equalTo(expected))
      },
      test("with nesting array") {
        val orc      = Orc(
          "Query" -> List("foo" -> Field.output.to("Foo")),
          "Foo"   -> List("bar" -> Field.output.to("Bar").asList),
          "Bar"   -> List("value" -> Field.output.to("Int"))
        )
        val schema   = render(orc)
        val expected = """|schema {
                          |  query: Query
                          |}
                          |
                          |type Bar {
                          |  value: Int
                          |}
                          |
                          |type Foo {
                          |  bar: [Bar]
                          |}
                          |
                          |type Query {
                          |  foo: Foo
                          |}""".stripMargin
        assertZIO(schema)(equalTo(expected))
      },
      suite("toType")(
        test("NamedType") {
          val tpe      = OrcBlueprint.toType(NamedType("Foo"))
          val expected = Blueprint.NamedType("Foo", false)
          assert(tpe)(equalTo(expected))
        },
        test("NamedType with List") {
          val tpe      = OrcBlueprint.toType(ListType(NonNull(NamedType("Foo"))))
          val expected = Blueprint.ListType(ofType = Blueprint.NamedType(name = "Foo", nonNull = true), nonNull = false)
          assert(tpe)(equalTo(expected))
        },
        test("NamedType with List nullable") {
          val tpe      = OrcBlueprint.toType(ListType(NamedType("Foo")))
          val expected = Blueprint
            .ListType(ofType = Blueprint.NamedType(name = "Foo", nonNull = false), nonNull = false)
          assert(tpe)(equalTo(expected))
        },
        test("nested non-null") {
          val tpe      = OrcBlueprint.toType(NonNull(NonNull(NonNull(NonNull(NamedType("Foo"))))))
          val expected = Blueprint.NamedType(name = "Foo", nonNull = true)
          assert(tpe)(equalTo(expected))
        },
        test("nested listType") {
          val tpe      = OrcBlueprint.toType(ListType(ListType(ListType(ListType(NamedType("Foo"))))))
          val expected = Blueprint.ListType(
            Blueprint.ListType(
              ofType = Blueprint.ListType(
                ofType = Blueprint
                  .ListType(ofType = Blueprint.NamedType(name = "Foo", nonNull = false), nonNull = false),
                nonNull = false
              ),
              nonNull = false
            ),
            nonNull = false
          )
          assert(tpe)(equalTo(expected))
        }
      )
    ).provide(GraphQLGenerator.live, TypeGenerator.live, StepGenerator.live, EvaluationRuntime.live)

  def render(orc: Orc): ZIO[GraphQLGenerator, Throwable, String] = orc.toBlueprint.flatMap(_.toGraphQL).map(_.render)
}
