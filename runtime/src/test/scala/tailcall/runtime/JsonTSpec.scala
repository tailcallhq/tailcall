package tailcall.runtime

import zio.Scope
import zio.json.ast.Json
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertTrue}

object JsonTSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JsonTransformationSpec")(
      test("constant") {
        val transformation = JsonT.const(Json.Num(1))
        val input: Json    = Json.Num(2)
        val expected: Json = Json.Num(1)
        assertTrue(transformation(input) == expected)
      },
      test("identity") {
        val transformation = JsonT.identity
        val input: Json    = Json.Num(2)
        val expected: Json = Json.Num(2)
        assertTrue(transformation(input) == expected)
      },
      test("toPair") {
        val transformation = JsonT.toPair
        val input: Json    = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(2))
        val expected: Json = Json.Arr(Json.Arr(Json.Str("a"), Json.Num(1)), Json.Arr(Json.Str("b"), Json.Num(2)))
        assertTrue(transformation(input) == expected)
      },
      test("applySpec") {
        val transformation = JsonT
          .applySpec("a" -> JsonT.const(Json.Num(1)), "b" -> JsonT.const(Json.Num(2)), "c" -> JsonT.identity)

        val input: Json    = Json.Obj("a" -> Json.Num(3), "b" -> Json.Num(4), "c" -> Json.Num(5))
        val expected: Json = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(2), "c" -> Json.Num(5))

        assertTrue(transformation(input) == expected)
      },
    )
}
