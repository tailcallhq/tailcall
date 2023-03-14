package tailcall.runtime

import tailcall.runtime.internal.DynamicValueUtil._
import tailcall.runtime.internal.{Caliban, Primitive}
import zio.json.ast.Json
import zio.schema.{DeriveSchema, DynamicValue, Schema, TypeId}
import zio.test._

import scala.collection.immutable.ListMap

object DynamicValueUtilSpec extends ZIOSpecDefault {
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

  final case class Foobar(foo: List[Int], bar: DynamicValue)

  object Foobar {
    implicit val schema: Schema[Foobar] = DeriveSchema.gen[Foobar]
  }

  override def spec =
    suite("DynamicValueUtilSpec")(
      test("asString") {
        check(Primitive.gen) { primitive =>
          assertTrue(asString(primitive.toDynamicValue) == Some(primitive.value.toString))
        } &&
        assertTrue(asString(DynamicValue(List(42))) == None)
      },
      test("toTyped") {
        assertTrue(toTyped[String](DynamicValue("Hello World!")) == Some("Hello World!")) &&
        assertTrue(toTyped[String](DynamicValue(42)) == None)
      },
      test("getPath") {
        val d = DynamicValue(Foobar(List(42), DynamicValue(Option(Map("baz" -> "Hello World!")))))
        assertTrue(getPath(d, List("foo", "0")) == Some(DynamicValue(42))) &&
        assertTrue(getPath(d, List("bar", "baz")) == Some(DynamicValue("Hello World!"))) &&
        assertTrue(getPath(d, List("foo", "1")) == None) &&
        assertTrue(getPath(d, List("bar", "qux")) == None) &&
        assertTrue(getPath(d, List("quux")) == None)
      },
      test("toResponseValue compose fromResponseValue == identity") {
        check(Caliban.genResponseValue) { responseValue =>
          assertTrue(toResponseValue(fromResponseValue(responseValue)) == responseValue)
        }
      },
      test("toInputValue compose fromInputValue == identity") {
        check(Caliban.genInputValue) { inputValue =>
          assertTrue(toInputValue(fromInputValue(inputValue)) == inputValue)
        }
      },
      test("record") {
        assertTrue(
          record("foo" -> DynamicValue(List(42)), "bar" -> DynamicValue("Hello World!")) == DynamicValue
            .Record(TypeId.Structural, ListMap("foo" -> DynamicValue(List(42)), "bar" -> DynamicValue("Hello World!")))
        )
      },
      test("toJson compose fromJson == identity")(check(genJson)(json => assertTrue(toJson(fromJson(json)) == json)))
    )
}
