package tailcall.runtime

import caliban.InputValue.ObjectValue
import caliban.Value
import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import tailcall.test.TailcallSpec
import zio.schema.DeriveSchema
import zio.schema.annotation.{caseName, fieldName}
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, assertTrue, assertZIO}
import zio.{Chunk, Scope}

object DirectiveCodecSpec extends TailcallSpec {
  import DirectiveCodec._

  @caseName("foo")
  final case class Foo(a: String, @fieldName("bee") b: Int)
  object Foo {
    implicit val codec: DirectiveCodec[Foo] = DirectiveCodec.fromSchema(DeriveSchema.gen[Foo])
  }

  @caseName("barBaz")
  sealed trait BarBaz
  object BarBaz {
    @caseName("bar")
    final case class Bar(a: String, b: Int) extends BarBaz

    @caseName("baz")
    final case class Baz(c: Boolean, d: Double) extends BarBaz
    implicit val codec: DirectiveCodec[BarBaz] = DirectiveCodec.fromSchema(DeriveSchema.gen[BarBaz])
  }

  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("DirectiveCodecSpec")(
      suite("case classes")(
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
        test("decoding with different name should fail") {
          val foo      = Foo("a", 1)
          val actual   = foo.toDirective.map(_.copy(name = "boo")).flatMap(_.fromDirective[Foo])
          val expected = Chunk(TValid.Cause("Expected directive name to be foo but was boo"))
          assertZIO(actual.toZIO.flip)(equalTo(expected))
        },
        test("name")(assertTrue(Foo.codec.name == "foo")),
      ),
      suite("sealed traits")(
        test("encoding should work") {
          val barBaz: BarBaz = BarBaz.Bar("a", 1)
          val directive      = barBaz.toDirective
          val expected       =
            Directive("barBaz", Map("bar" -> ObjectValue(Map("a" -> Value.StringValue("a"), "b" -> Value.IntValue(1)))))
          assertZIO(directive.toZIO)(equalTo(expected))
        },
        test("name")(assertTrue(BarBaz.codec.name == "barBaz")),
        test("decoding should work") {
          val barBaz: BarBaz = BarBaz.Bar("a", 1)
          val actual         = barBaz.toDirective.flatMap(_.fromDirective[BarBaz])
          val expected       = barBaz
          assertZIO(actual.toZIO)(equalTo(expected))
        },
      ),
    )
}
