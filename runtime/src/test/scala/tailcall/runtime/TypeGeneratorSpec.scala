package tailcall.runtime

import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.scala.Orc.Field
import tailcall.runtime.dsl.scala.Orc.Type.{ListType, NamedType, NonNull}
import tailcall.runtime.dsl.scala.{Orc, OrcBlueprint}
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio.ZIO
import zio.http.Client
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
      ),
      suite("mutation")(
        test("mutation with primitive input") {
          // mutation createFoo(input: String){foo: Foo}
          // type Foo {a: Int, b: Int, c: Int}
          val orc    = Orc(
            "Query"    -> List("foo" -> Field.output.to("Foo").resolveWith(Map("a" -> 1))),
            "Foo"      -> List("a" -> Field.output.to("Int")),
            "Mutation" -> List("createFoo" -> Field.output.to("Foo").withArgument("input" -> Field.input.to("String")))
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
          // mutation createFoo(input: FooInput){foo: String}
          // input FooInput {a: Int, b: Int, c: Int}
          val orc    = Blueprint(
            Blueprint.SchemaDefinition(query = Option("Query"), mutation = Option("Mutation"), subscription = None),
            List(
              Blueprint.ObjectTypeDefinition(
                name = "Query",
                fields = List(Blueprint.FieldDefinition(name = "foo", Nil, Blueprint.NamedType("Foo", false)))
              ),
              Blueprint.ObjectTypeDefinition(
                name = "Mutation",
                fields = List(Blueprint.FieldDefinition(
                  name = "createFoo",
                  List(Blueprint.InputValueDefinition(name = "input", Blueprint.NamedType("FooInput", false), None)),
                  Blueprint.NamedType("Foo", false)
                ))
              ),
              Blueprint.ObjectTypeDefinition(
                name = "Foo",
                fields = List(Blueprint.FieldDefinition(name = "a", Nil, Blueprint.NamedType("Int", false)))
              ),
              Blueprint.InputObjectTypeDefinition(
                name = "FooInput",
                fields = List(Blueprint.InputValueDefinition(name = "a", Blueprint.NamedType("Int", false), None))
              )
            )
          )
          val schema = orc.toGraphQL.map(_.render)
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
                                       |type Query {
                                       |  foo: Foo
                                       |}""".stripMargin))
        }
      )
    ).provide(
      GraphQLGenerator.live,
      TypeGenerator.live,
      StepGenerator.live,
      EvaluationRuntime.live,
      HttpClient.live,
      Client.default
    )

  def render(orc: Orc): ZIO[GraphQLGenerator, Throwable, String] = orc.toBlueprint.flatMap(_.toGraphQL).map(_.render)
}
