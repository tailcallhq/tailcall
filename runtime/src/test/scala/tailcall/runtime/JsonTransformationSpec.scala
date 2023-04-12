package tailcall.runtime

import zio.Scope
import zio.json.ast.Json
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertTrue}

object JsonTransformationSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JsonTransformationSpec")(
      test("constant") {
        val transformation = JsonTransformation.const(Json.Num(1))
        val input: Json    = Json.Num(2)
        val expected: Json = Json.Num(1)
        assertTrue(transformation(input) == expected)
      },
      test("identity") {
        val transformation = JsonTransformation.identity
        val input: Json    = Json.Num(2)
        val expected: Json = Json.Num(2)
        assertTrue(transformation(input) == expected)
      },
      test("toPair") {
        val transformation = JsonTransformation.toPair
        val input: Json    = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(2))
        val expected: Json = Json.Arr(Json.Arr(Json.Str("a"), Json.Num(1)), Json.Arr(Json.Str("b"), Json.Num(2)))
        assertTrue(transformation(input) == expected)
      },
      test("applySpec") {
        val transformation = JsonTransformation.applySpec(
          "a" -> JsonTransformation.const(Json.Num(1)),
          "b" -> JsonTransformation.const(Json.Num(2)),
          "c" -> JsonTransformation.identity,
        )

        val input: Json    = Json.Obj("a" -> Json.Num(3), "b" -> Json.Num(4), "c" -> Json.Num(5))
        val expected: Json = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(2), "c" -> Json.Num(5))

        assertTrue(transformation(input) == expected)
      },
    )
}
