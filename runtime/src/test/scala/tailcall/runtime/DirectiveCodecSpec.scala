package tailcall.runtime

import caliban.Value
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec.{DecoderSyntax, EncoderSyntax}
import zio.Scope
import zio.json.jsonHint
import zio.schema.DeriveSchema
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertZIO}

object DirectiveCodecSpec extends ZIOSpecDefault {
  @jsonHint("foo")
  final private case class Foo(a: String, @jsonHint("bee") b: Int)
  implicit private val schema                     = DeriveSchema.gen[Foo]
  implicit private val codec: DirectiveCodec[Foo] = DirectiveCodec.fromSchema(schema)

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("DirectiveCodecSpec")(
      test("encoding should work") {
        val foo       = Foo("a", 1)
        val directive = foo.toDirective
        val expected  = Directive("foo", Map("a" -> Value.StringValue("a"), "bee" -> Value.IntValue(1)))
        assertZIO(directive.toZIO)(equalTo(expected))
      },
      test("decoding should work") {
        val foo      = Foo("a", 1)
        val actual   = foo.toDirective.flatMap(_.fromDirective[Foo])
        val expected = foo
        assertZIO(actual.toZIO)(equalTo(expected))
      },
    )
}
