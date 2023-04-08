package tailcall.runtime

import caliban.Value.StringValue
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec.{DecoderSyntax, EncoderSyntax}
import tailcall.runtime.model.FieldUpdateAnnotation
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

object FieldUpdateAnnotationSpec extends ZIOSpecDefault {
  override def spec =
    suite("FieldUpdateAnnotationSpec")(suite("directive")(
      test("encoding") {
        val rename: FieldUpdateAnnotation = FieldUpdateAnnotation.empty.withName("foo")
        val actual                        = rename.toDirective
        val expected                      = Directive("update", arguments = Map("rename" -> StringValue("foo")))
        assertZIO(actual.toZIO)(equalTo(expected))
      },
      test("decoding") {
        val directive = Directive("update", arguments = Map("rename" -> StringValue("foo")))
        val actual    = directive.fromDirective[FieldUpdateAnnotation]
        val expected  = FieldUpdateAnnotation.empty.withName("foo")
        assertZIO(actual.toZIO)(equalTo(expected))
      },
    ))
}
