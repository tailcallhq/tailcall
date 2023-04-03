package tailcall.runtime

import tailcall.runtime.model.TSchema
import tailcall.runtime.transcoder.JsonValue2TSchema
import zio.Scope
import zio.json.ast.Json
import zio.test.Assertion.{equalTo, isNone, isSome}
import zio.test._

object JsonValue2TSchemaSpec extends ZIOSpecDefault with JsonValue2TSchema {
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    import tailcall.runtime.SchemaUnifier.unify
    suite("json to TSchema")(
      suite("unify")(
        test("removes duplicates") {
          val schema = unify(TSchema.int, TSchema.int)
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.int)))
        },
        test("objects") {
          val schema = unify(TSchema.obj("a" -> TSchema.int), TSchema.obj("b" -> TSchema.int))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.int.opt, "b" -> TSchema.int.opt))))
        },
        test("array") {
          val schema = unify(TSchema.obj("a" -> TSchema.int).arr, TSchema.obj("b" -> TSchema.int).arr)
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.int.opt, "b" -> TSchema.int.opt).arr)))
        },
        test("optional(a) b") {
          val schema = unify(TSchema.obj("a" -> TSchema.int.opt), TSchema.obj("a" -> TSchema.int))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.int.opt))))
        },
        test("a optional(b)") {
          val schema = unify(TSchema.obj("a" -> TSchema.int), TSchema.obj("a" -> TSchema.int.opt))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.int.opt))))
        },
        test("optional(a) optional(b)") {
          val schema = unify(TSchema.obj("a" -> TSchema.int.opt), TSchema.obj("a" -> TSchema.int.opt))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.int.opt))))
        },
        test("int and string") {
          val schema = unify(TSchema.int, TSchema.string)
          assertZIO(schema.toZIO)(isNone)
        },
        test("deeply nested") {
          val schema = unify(
            TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("x" -> TSchema.int))),
            TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("y" -> TSchema.int))),
          )
          assertZIO(schema.toZIO)(isSome(
            equalTo(TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("x" -> TSchema.int.opt, "y" -> TSchema.int.opt))))
          ))
        },
      ),
      suite("json to TSchema")(
        test("object to tSchema") {
          val json     = """{"a": 1, "b": "2", "c": true, "d": null, "e": [1, 2, 3]}"""
          val expected = TSchema.obj(
            "a" -> TSchema.Int,
            "b" -> TSchema.String,
            "c" -> TSchema.Boolean,
            "d" -> TSchema.empty,
            "e" -> TSchema.arr(TSchema.Int),
          )
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("array tSchema") {
          val json     = Json.Arr(
            Json.Obj("a" -> Json.Num(1), "b" -> Json.Bool(true)),
            Json.Obj("a" -> Json.Num(1), "b" -> Json.Bool(true)),
            Json.Obj("a" -> Json.Num(2), "b" -> Json.Bool(false)),
          )
          val expected = TSchema.arr(TSchema.obj("a" -> TSchema.Int, "b" -> TSchema.Boolean))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("nullables to TSchema") {
          val json     = Json.Arr(Json.Obj("a" -> Json.Num(1)), Json.Obj("a" -> Json.Null))
          val expected = TSchema.arr(TSchema.obj("a" -> (TSchema.Int.opt)))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("nullables with multiple keys to TSchema") {
          val json     = Json
            .Arr(Json.Obj("a" -> Json.Num(1), "b" -> Json.Null), Json.Obj("a" -> Json.Null, "b" -> Json.Num(1)))
          val expected = TSchema.arr(TSchema.obj("a" -> TSchema.int.opt, "b" -> TSchema.int.opt))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("numeric keys") {
          val json     = Json.Obj("1" -> Json.Num(1), "2" -> Json.Str("1"), "3" -> Json.Bool(true))
          val expected = TSchema.obj("_1" -> TSchema.Int, "_2" -> TSchema.String, "_3" -> TSchema.Boolean)
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
      ),
    )
  }
}
