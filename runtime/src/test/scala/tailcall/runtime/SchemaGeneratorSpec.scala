package tailcall.runtime

import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.scala.Orc
import tailcall.runtime.dsl.scala.Orc.Type.{ListType, NamedType, NonNull}
import tailcall.runtime.dsl.scala.Orc.{Field, FieldSet}
import tailcall.runtime.service._
import zio.ZIO
import zio.test.Assertion._
import zio.test._

object SchemaGeneratorSpec extends ZIOSpecDefault {
  override def spec =
    suite("SchemaGenerator")(
      test("document type generation") {
        val orc = Orc("Query" -> FieldSet("test" -> Field.output.to("String").resolveWith("test")))

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
          "Query" -> FieldSet(
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
          "Query" -> FieldSet(
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
          "Query" -> FieldSet("foo" -> Field.output.to("Foo")),
          "Foo"   -> FieldSet("bar" -> Field.output.to("Bar")),
          "Bar"   -> FieldSet("value" -> Field.output.to("Int").resolveWith(100))
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
          "Query" -> FieldSet("foo" -> Field.output.to("Foo")),
          "Foo"   -> FieldSet("bar" -> Field.output.to("Bar").asList),
          "Bar"   -> FieldSet("value" -> Field.output.to("Int"))
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
      ),
      suite("mutation")(
        test("mutation with primitive input") {
          // mutation createFoo(input: String){foo: Foo}
          // type Foo {a: Int, b: Int, c: Int}
          val orc = Orc(
            "Query"    -> FieldSet("foo" -> Field.output.to("Foo").resolveWith(Map("a" -> 1))),
            "Foo"      -> FieldSet("a" -> Field.output.to("Int")),
            "Mutation" -> FieldSet(
              "createFoo" -> Field.output.to("Foo").withArgument("input" -> Field.input.to("String"))
            )
          )

          val schema = render(orc)
          assertZIO(schema)(equalTo("""|schema {
                                       |  query: Query
                                       |  mutation: Mutation
                                       |}
                                       |
                                       |type Foo {
                                       |  a: Int
                                       |}
                                       |
                                       |type Mutation {
                                       |  createFoo(input: String): Foo
                                       |}
                                       |
                                       |type Query {
                                       |  foo: Foo
                                       |}""".stripMargin))
        },
        test("mutation with input type") {
          // schema {mutation: Mutation}
          // type Mutation { createFoo(input: FooInput) Foo }
          // type Foo { foo: String }
          // input FooInput {a: Int, b: Int, c: Int}

          val orc = Orc(
            "Query"    -> FieldSet.Empty,
            "Mutation" -> FieldSet(
              "createFoo" -> Field.output.to("Foo").withArgument("input" -> Field.input.to("FooInput"))
            ),
            "Foo"      -> FieldSet("a" -> Field.output.to("Int")),
            "FooInput" -> FieldSet("a" -> Field.input.to("Int"))
          )

          val schema = orc.toBlueprint.flatMap(_.toGraphQL).map(_.render)
          assertZIO(schema)(equalTo("""|schema {
                                       |  query: Query
                                       |  mutation: Mutation
                                       |}
                                       |
                                       |input FooInput {
                                       |  a: Int
                                       |}
                                       |
                                       |type Foo {
                                       |  a: Int
                                       |}
                                       |
                                       |type Mutation {
                                       |  createFoo(input: FooInput): Foo
                                       |}
                                       |
                                       |type Query""".stripMargin))
        }
      )
    ).provide(GraphQLGenerator.live, SchemaGenerator.live, StepGenerator.live, EvaluationRuntime.live)

  def render(orc: Orc): ZIO[GraphQLGenerator, Throwable, String] = orc.toBlueprint.flatMap(_.toGraphQL).map(_.render)
}
