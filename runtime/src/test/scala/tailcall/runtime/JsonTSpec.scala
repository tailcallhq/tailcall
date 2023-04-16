package tailcall.runtime

import zio.Scope
import zio.json.{DecoderOps, EncoderOps}
import zio.json.ast.Json
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertTrue}

object JsonTSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JsonTSpec")(
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
      test("toKeyValue") {
        val transformation = JsonT.toKeyValue
        val input: Json    = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(2))
        val expected: Json = Json.Arr(
          Json.Obj("key" -> Json.Str("a"), "value" -> Json.Num(1)),
          Json.Obj("key" -> Json.Str("b"), "value" -> Json.Num(2)),
        )
        assertTrue(transformation(input) == expected)
      },
      test("applySpec") {
        val transformation = JsonT.applySpec("a" -> JsonT.identity, "b" -> JsonT.const(Json.Str("b")))
        val input: Json    = Json.Num(1)
        val expected: Json = Json.Obj("a" -> Json.Num(1), "b" -> Json.Str("b"))
        assertTrue(transformation(input) == expected)
      },
      test("objectPath") {
        val transformation = JsonT.objPath("x" -> List("a", "b", "c"))
        val input: Json    = Json.Obj("a" -> Json.Obj("b" -> Json.Obj("c" -> Json.Num(1))))
        val expected: Json = Json.Obj("x" -> Json.Num(1))
        assertTrue(transformation(input) == expected)
      },
      test("map") {
        val transformation = JsonT.map(JsonT.path("a"))
        val input: Json    = Json.Arr(Json.Obj("a" -> Json.Num(1)), Json.Obj("a" -> Json.Num(2)))
        val expected: Json = Json.Arr(Json.Num(1), Json.Num(2))
        assertTrue(transformation(input) == expected)
      },
      test("invalid map") {
        val transformation = JsonT.map(JsonT.path("a"))
        val input: Json    = Json.Num(1)
        val expected: Json = Json.Arr()
        assertTrue(transformation(input) == expected)
      },
      test("compose") {
        val transformation = JsonT.compose(JsonT.path("a"), JsonT.path("b"))
        val input: Json    = Json.Obj("b" -> Json.Obj("a" -> Json.Num(1)))
        val expected: Json = Json.Num(1)
        assertTrue(transformation(input) == expected)
      },
      test("pipe") {
        val transformation = JsonT.pipe(JsonT.path("b"), JsonT.path("a"))
        val input: Json    = Json.Obj("b" -> Json.Obj("a" -> Json.Num(1)))
        val expected: Json = Json.Num(1)
        assertTrue(transformation(input) == expected)
      },
      test("omit") {
        val transformation = JsonT.omit("x", "y")
        val input: Json    = Json.Obj("x" -> Json.Num(1), "y" -> Json.Num(2), "z" -> Json.Num(3))
        val expected: Json = Json.Obj("z" -> Json.Num(3))
        assertTrue(transformation(input) == expected)
      },
      test("flatMap") {
        val transformation = JsonT.flatMap(JsonT.path("a"))
        val input: Json    = Json.Arr(Json.Obj("a" -> Json.Arr(Json.Num(1))), Json.Obj("a" -> Json.Arr(Json.Num(2))))
        val expected: Json = Json.Arr(Json.Num(1), Json.Num(2))
        assertTrue(transformation(input) == expected)
      },
      test("transform composition") {
        val input          = """{
                      |  "value": {
                      |    "data": [
                      |      {
                      |        "observations": {
                      |          "diagnosis": {
                      |            "0": {
                      |              "text": "Yes yes Acne also",
                      |              "pretext": [
                      |                2020,
                      |                9393
                      |              ],
                      |              "metadata": {
                      |                "type": "final"
                      |              }
                      |            },
                      |            "pretext": []
                      |          }
                      |        }
                      |      }
                      |    ]
                      |  }
                      |}""".stripMargin
        val inputJson      = input.fromJson[Json].getOrElse(Json.Null)
        val transformation = JsonT.compose(
          JsonT
            .flatMap(JsonT.compose(JsonT.toKeyValue, JsonT.omit("pretext"), JsonT.path("observations", "diagnosis"))),
          JsonT.path("value", "data"),
        )
        // val transformation =
        assertTrue(transformation(inputJson).toJsonPretty == """[
                                                               |  {
                                                               |    "key" : "0",
                                                               |    "value" : {
                                                               |      "text" : "Yes yes Acne also",
                                                               |      "pretext" : [
                                                               |        2020,
                                                               |        9393
                                                               |      ],
                                                               |      "metadata" : {
                                                               |        "type" : "final"
                                                               |      }
                                                               |    }
                                                               |  }
                                                               |]""".stripMargin)
      },
    )
}
