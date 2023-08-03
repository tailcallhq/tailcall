package tailcall.runtime

import caliban.Value.StringValue
import caliban.parsing.adt.Directive
import tailcall.TailcallSpec
import tailcall.runtime.DirectiveCodec.{DecoderSyntax, EncoderSyntax}
import tailcall.runtime.model.ModifyField
import zio.test.Assertion.equalTo
import zio.test.assertZIO

object ModifyFieldSpec extends TailcallSpec {
  override def spec =
    suite("FieldUpdateAnnotationSpec")(suite("directive")(
      test("encoding") {
        val rename: ModifyField = ModifyField.empty.withName("foo")
        val actual              = rename.toDirective
        val expected            = Directive("modify", arguments = Map("name" -> StringValue("foo")))
        assertZIO(actual.toZIO)(equalTo(expected))
      },
      test("decoding") {
        val directive = Directive("modify", arguments = Map("name" -> StringValue("foo")))
        val actual    = directive.fromDirective[ModifyField]
        val expected  = ModifyField.empty.withName("foo")
        assertZIO(actual.toZIO)(equalTo(expected))
      },
    ))
}
