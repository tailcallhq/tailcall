package tailcall.runtime

import caliban.InputValue
import caliban.Value.StringValue
import caliban.parsing.adt.Directive
import tailcall.TailcallSpec
import tailcall.runtime.DirectiveCodec.{DecoderSyntax, EncoderSyntax}
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.InlineType
import zio.Scope
import zio.test.{Spec, TestEnvironment, assertTrue}

object InlineTypeSpec extends TailcallSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("InlineType")(
      test("toDirective") {
        val inline    = InlineType(List("a", "b", "c"))
        val directive = inline.toDirective
        val expected  = TValid.succeed(Directive(
          "inline",
          Map("path" -> InputValue.ListValue(List(StringValue("a"), StringValue("b"), StringValue("c")))),
        ))

        assertTrue(directive == expected)
      },
      test("fromDirective") {
        val directive = Directive(
          "inline",
          Map("path" -> InputValue.ListValue(List(StringValue("a"), StringValue("b"), StringValue("c")))),
        )

        val actual   = directive.fromDirective[InlineType]
        val expected = TValid.succeed(InlineType(List("a", "b", "c")))

        assertTrue(actual == expected)
      },
    )
}
