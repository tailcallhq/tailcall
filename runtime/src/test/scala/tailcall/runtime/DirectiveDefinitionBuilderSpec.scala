package tailcall.runtime

import caliban.parsing.adt.Definition.TypeSystemDefinition.DirectiveDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.DirectiveLocation.TypeSystemDirectiveLocation.FIELD_DEFINITION
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.InputValueDefinition
import caliban.parsing.adt.Type
import zio.Scope
import zio.schema.annotation.caseName
import zio.schema.{DeriveSchema, Schema}
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertTrue}

object DirectiveDefinitionBuilderSpec extends ZIOSpecDefault {
  @caseName("foo")
  final case class Foo(name: String, age: Int)
  object Foo {
    implicit val schema: Schema[Foo] = DeriveSchema.gen[Foo]
  }
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("DirectiveDefinitionBuilderSpec")(
      test("Unit") {
        val directiveDefinition = DirectiveDefinitionBuilder.make[Unit]
        val expected            = DirectiveDefinition(None, "Unit", Nil, Set.empty)
        assertTrue(directiveDefinition.unsafeBuild == expected)
      },
      test("withLocations") {
        val directiveDefinition = DirectiveDefinitionBuilder.make[Unit].withLocations(FIELD_DEFINITION)
        val expected            = DirectiveDefinition(None, "Unit", Nil, Set(FIELD_DEFINITION))
        assertTrue(directiveDefinition.unsafeBuild == expected)
      },
      test("withDescription") {
        val directiveDefinition = DirectiveDefinitionBuilder.make[Unit].withDescription("description")
        val expected            = DirectiveDefinition(Some("description"), "Unit", Nil, Set.empty)
        assertTrue(directiveDefinition.unsafeBuild == expected)
      },
      test("withType") {
        val directiveDefinition = DirectiveDefinitionBuilder.make[Foo]
        val args                = List(
          InputValueDefinition(
            name = "name",
            description = None,
            ofType = Type.NamedType("String", nonNull = true),
            defaultValue = None,
            directives = Nil,
          ),
          InputValueDefinition(
            name = "age",
            description = None,
            ofType = Type.NamedType("Int", nonNull = true),
            defaultValue = None,
            directives = Nil,
          ),
        )
        val expected            = DirectiveDefinition(None, "foo", args, Set.empty)
        assertTrue(directiveDefinition.unsafeBuild == expected)
      },
    )
  }
}
