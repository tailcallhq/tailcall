package tailcall.runtime

import caliban.{ResponseValue, Value}
import tailcall.runtime.internal.DynamicValueUtil._
import tailcall.runtime.internal.{Caliban, Primitive}
import zio.json.ast.Json
import zio.schema.{DeriveSchema, DynamicValue, Schema, TypeId}
import zio.test._

import scala.collection.immutable.ListMap
import scala.util.Try

object DynamicValueUtilSpec extends ZIOSpecDefault {
  val helloWorld    = "Hello World!"
  val meaningOfLife = 42

  val genJson: Gen[Any, Json] = Gen.suspend(Gen.oneOf(
    Gen.chunkOfBounded(0, 5)(for {
      key   <- Gen.string1(Gen.alphaChar)
      value <- genJson
    } yield (key, value)).map(Json.Obj(_)),
    Gen.chunkOfBounded(0, 5)(genJson).map(Json.Arr(_)),
    Gen.boolean.map(Json.Bool(_)),
    Gen.string.map(Json.Str(_)),
    Gen.double.map(Json.Num(_)),
    Gen.const(Json.Null)
  ))

  sealed trait Foo

  final case class Foobar(foo: List[Int], bar: DynamicValue)

  object Foo {
    final case class Bar(bar: List[Option[Int]])        extends Foo
    final case class Baz(baz: Set[Either[Int, String]]) extends Foo

    implicit val schema: Schema[Foo] = DeriveSchema.gen[Foo]
  }

  object Foobar {
    implicit val schema: Schema[Foobar] = DeriveSchema.gen[Foobar]
  }

  override def spec =
    suite("DynamicValueUtilSpec")(
      test("asString") {
        check(Primitive.gen) { primitive =>
          assertTrue(asString(primitive.toDynamicValue) == Some(primitive.value.toString))
        } &&
        assertTrue(asString(DynamicValue(List(meaningOfLife))) == None)
      },
      test("toResponseValue") {
        assertTrue(
          toResponseValue(DynamicValue(Foobar(List(meaningOfLife), DynamicValue(())))) == ResponseValue
            .ObjectValue(List("foo" -> ResponseValue.ListValue(List(Value.IntValue(42))), "bar" -> Value.NullValue))
        ) &&
        assertTrue(
          toResponseValue(
            DynamicValue(Map("foo" -> DynamicValue(List(meaningOfLife)), "bar" -> DynamicValue(())))
          ) == ResponseValue
            .ObjectValue(List("foo" -> ResponseValue.ListValue(List(Value.IntValue(42))), "bar" -> Value.NullValue))
        ) &&
        assert(Try(toResponseValue(DynamicValue(Map(meaningOfLife -> helloWorld)))))(Assertion.isFailure(
          Assertion.hasMessage(Assertion.equalTo("could not transform"))
        ))
      },
      test("toTyped") {
        assertTrue(toTyped[String](DynamicValue(helloWorld)) == Some("Hello World!")) &&
        assertTrue(toTyped[String](DynamicValue(meaningOfLife)) == None)
      },
      test("getPath") {
        val d = DynamicValue(Foobar(List(meaningOfLife), DynamicValue(Option(Map("baz" -> helloWorld)))))
        assertTrue(getPath(d, List("foo", "0")) == Some(DynamicValue(42))) &&
        assertTrue(getPath(d, List("bar", "baz")) == Some(DynamicValue("Hello World!"))) &&
        assertTrue(getPath(d, List("foo", "1")) == None) &&
        assertTrue(getPath(d, List("bar", "qux")) == None) &&
        assertTrue(getPath(d, List("quux")) == None)
      },
      test("toInputValue compose fromInputValue == identity") {
        check(Caliban.genInputValue) { inputValue =>
          assertTrue(toInputValue(fromInputValue(inputValue)) == inputValue)
        }
      },
      test("record") {
        assertTrue(
          record("foo" -> DynamicValue(List(meaningOfLife)), "bar" -> DynamicValue(helloWorld)) == DynamicValue
            .Record(TypeId.Structural, ListMap("foo" -> DynamicValue(List(42)), "bar" -> DynamicValue("Hello World!")))
        )
      },
      test("toJson compose fromJson == identity")(check(genJson)(json => assertTrue(toJson(fromJson(json)) == json)))
    )
}
