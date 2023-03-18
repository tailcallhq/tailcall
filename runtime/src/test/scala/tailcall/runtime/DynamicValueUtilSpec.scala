package tailcall.runtime

import caliban.{InputValue, ResponseValue}
import tailcall.runtime.internal.DynamicValueUtil._
import tailcall.runtime.internal.{CalibanGen, JsonGen, PrimitiveGen}
import tailcall.runtime.transcoder.Transcoder.Syntax
import zio.json.ast.Json
import zio.schema.DynamicValue
import zio.test._

object DynamicValueUtilSpec extends ZIOSpecDefault {
  override def spec =
    suite("DynamicValueUtilSpec")(
      suite("asString")(
        test("valid") {
          val dynamics: Gen[Any, (DynamicValue, String)] = Gen.oneOf(PrimitiveGen.genPrimitive.map { primitive =>
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
            DynamicValue(Map("a" -> Option(Map("b" -> 1))))     -> List("a", "b")      -> 1,
            DynamicValue(Map("a" -> Map("b" -> Map("c" -> 1)))) -> List("a", "b", "c") -> 1,
            DynamicValue(Map("a" -> List(Map("b" -> 1))))       -> List("a", "0", "b") -> 1,
            record("a" -> DynamicValue(1))                      -> List("a")           -> 1
          ))

          checkAll(gen) { case dynamic -> path -> expected =>
            assertTrue(getPath(dynamic, path) == Some(DynamicValue(expected)))
          }
        },
        test("invalid") {
          val gen = Gen.fromIterable(Seq(
            DynamicValue(Map("a" -> 1))                         -> List("b"),
            DynamicValue(Map("a" -> Map("b" -> 1)))             -> List("b", "b"),
            DynamicValue(Map("a" -> Option(Map("b" -> 1))))     -> List("a", "c"),
            DynamicValue(Map("a" -> Map("b" -> Map("c" -> 1)))) -> List("a", "c", "e"),
            DynamicValue(Map("a" -> List(Map("b" -> 1))))       -> List("a", "1", "b"),
            record("a" -> DynamicValue(1))                      -> List("d")
          ))

          checkAll(gen) { case dynamic -> path => assertTrue(getPath(dynamic, path) == None) }
        }
      ),
      test("fromResponseValue >>> toResponseValue == identity") {
        check(CalibanGen.genResponseValue) { responseValue =>
          val actual   = responseValue.transcode[DynamicValue].transcode[ResponseValue]
          val expected = responseValue
          assertTrue(actual == expected)
        }
      },
      test("fromInputValue >>> toInputValue == identity") {
        check(CalibanGen.genInputValue) { inputValue =>
          val actual   = inputValue.transcode[DynamicValue].transcode[InputValue]
          val expected = inputValue
          assertTrue(actual == expected)
        }
      },
      test("fromJson >>> toJson == identity")(check(JsonGen.genJson)(json => {
        val actual   = json.transcode[DynamicValue].transcodeOrFailWith[Json, String]
        val expected = Right(json)
        assertTrue(actual == expected)
      }))
    )
}
