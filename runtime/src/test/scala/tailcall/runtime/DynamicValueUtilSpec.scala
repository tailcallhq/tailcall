package tailcall.runtime

import tailcall.runtime.internal.CalibanGen
import tailcall.runtime.internal.DynamicValueUtil._
import tailcall.runtime.internal.PrimitiveGen.Primitive
import zio.json.ast.Json
import zio.schema.{DeriveSchema, DynamicValue, Schema}
import zio.test._

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
      suite("asString")(
        test("valid") {
          val dynamics: Gen[Any, (DynamicValue, String)] = Gen.oneOf(Primitive.gen.map { primitive =>
            primitive.toDynamicValue -> primitive.value.toString
          })

          checkAll(dynamics) { case (dynamic, expected) => assertTrue(asString(dynamic) == Some(expected)) }
        },
        test("invalid") {
          val dynamics: Gen[Any, DynamicValue] = Gen.fromIterable(Seq(DynamicValue(List(42)), DynamicValue(Option(1))))
          checkAll(dynamics)(dynamic => assertTrue(asString(dynamic) == None))
        }
      ),
      suite("toTyped")(
        test("valid") {
          val gen = Gen.fromIterable(Seq(
            toTyped[String](DynamicValue("Hello World!")) -> Some("Hello World!"),
            toTyped[Int](DynamicValue(42))                -> Some(42)
          ))

          checkAll(gen) { case (dynamicValue, expected) => assertTrue(dynamicValue == expected) }
        },
        test("invalid") {
          val gen = Gen.fromIterable(Seq(toTyped[Int](DynamicValue("Hello World!")), toTyped[String](DynamicValue(42))))

          checkAll(gen)(dynamicValue => assertTrue(dynamicValue == None))
        }
      ),
      suite("getPath")(
        test("valid") {
          val gen = Gen.fromIterable(Seq(
            DynamicValue(Map("a" -> 1))                         -> List("a")           -> 1,
            DynamicValue(Map("a" -> Map("b" -> 1)))             -> List("a", "b")      -> 1,
            DynamicValue(Map("a" -> Map("b" -> Map("c" -> 1)))) -> List("a", "b", "c") -> 1,
            DynamicValue(Map("a" -> List(Map("b" -> 1))))       -> List("a", "0", "b") -> 1,
            record("a" -> DynamicValue(1))                      -> List("a")           -> 1
            // TODO: options

          ))

          checkAll(gen) { case dynamic -> path -> expected =>
            assertTrue(getPath(dynamic, path) == Some(DynamicValue(expected)))
          }
        },
        test("invalid")(assertCompletes)
      ),
      test("fromResponseValue >=> toResponseValue == Option") {
        check(CalibanGen.genResponseValue) { responseValue =>
          val actual   = fromResponseValue(responseValue).flatMap(toResponseValue)
          val expected = Option(responseValue)
          assertTrue(actual == expected)
        }
      },
      test("fromInputValue >=> toInputValue == Option") {
        check(CalibanGen.genInputValue) { inputValue =>
          val actual   = fromInputValue(inputValue).flatMap(toInputValue)
          val expected = Option(inputValue)
          assertTrue(actual == expected)
        }
      },
      test("fromJson >>> toJson == Option")(check(genJson)(json => {
        val actual   = toJson(fromJson(json))
        val expected = Option(json)
        assertTrue(actual == expected)
      }))
    )
}
