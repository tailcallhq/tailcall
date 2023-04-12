package tailcall.runtime

import zio.Scope
import zio.json.ast.Json
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertTrue}

object JsonTransformationSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JsonTransformationSpec")(
      test("constant") {
        val transformation: JsonTransformation[Json] = JsonTransformation.Constant(Json.Num(1))
        val input                                    = Json.Num(2)
        val expected                                 = Json.Num(1)
        assertTrue(transformation(input) == expected)
      },
      test("identity") {
        val transformation: JsonTransformation[Json] = JsonTransformation.Identity()
        val input                                    = Json.Num(2)
        val expected                                 = Json.Num(2)
        assertTrue(transformation(input) == expected)
      },
      test("toPair") {
        val transformation = JsonTransformation.ToPair[Json]()
        val input          = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(2))
        val expected       = Json.Arr(Json.Arr(Json.Str("a"), Json.Num(1)), Json.Arr(Json.Str("b"), Json.Num(2)))
        assertTrue(transformation(input) == expected)
      },
      test("applySpec") {
        val transformation = JsonTransformation.ApplySpec[Json](Map(
          "a" -> JsonTransformation.Constant(Json.Num(1)),
          "b" -> JsonTransformation.Constant(Json.Num(2)),
          "c" -> JsonTransformation.Identity(),
        ))

        val input    = Json.Obj("a" -> Json.Num(3), "b" -> Json.Num(4), "c" -> Json.Num(5))
        val expected = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(2), "c" -> Json.Num(5))

        assertTrue(transformation(input) == expected)
      },
    )
}
