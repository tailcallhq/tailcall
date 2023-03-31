package tailcall.runtime

import tailcall.runtime.ast.TSchema
import tailcall.runtime.transcoder.JsonValue2TSchema
import zio.Scope
import zio.json.ast.Json
import zio.test.Assertion.equalTo
import zio.test._

object JsonValue2TSchemaSpec extends ZIOSpecDefault with JsonValue2TSchema {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("json to TSchema")(
      suite("unify")(
        test("removes duplicates") {
          val schema = unify(TSchema.int, TSchema.int)
          assertZIO(schema.toZIO)(equalTo(TSchema.int))
        },
        test("objects") {
          val schema = unify(TSchema.obj("a" -> TSchema.int), TSchema.obj("b" -> TSchema.int))
          assertZIO(schema.toZIO)(equalTo(TSchema.obj("a" -> TSchema.int.opt, "b" -> TSchema.int.opt)))
        },
        test("array") {
          val schema = unify(TSchema.obj("a" -> TSchema.int).arr, TSchema.obj("b" -> TSchema.int).arr)
          assertZIO(schema.toZIO)(equalTo(TSchema.obj("a" -> TSchema.int.opt, "b" -> TSchema.int.opt).arr))
        },
        test("optional(a) b") {
          val schema = unify(TSchema.obj("a" -> TSchema.int.opt), TSchema.obj("a" -> TSchema.int))
          assertZIO(schema.toZIO)(equalTo(TSchema.obj("a" -> TSchema.int.opt)))
        },
        test("a optional(b)") {
          val schema = unify(TSchema.obj("a" -> TSchema.int), TSchema.obj("a" -> TSchema.int.opt))
          assertZIO(schema.toZIO)(equalTo(TSchema.obj("a" -> TSchema.int.opt)))
        },
        test("optional(a) optional(b)") {
          val schema = unify(TSchema.obj("a" -> TSchema.int.opt), TSchema.obj("a" -> TSchema.int.opt))
          assertZIO(schema.toZIO)(equalTo(TSchema.obj("a" -> TSchema.int.opt)))
        },
        test("int and string") {
          val schema = unify(TSchema.int, TSchema.string)
          assertZIO(schema.toZIO)(equalTo(TSchema.string))
        },
        test("deeply nested") {
          val schema = unify(
            TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("x" -> TSchema.int))),
            TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("y" -> TSchema.int))),
          )
          assertZIO(schema.toZIO)(equalTo(
            TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("x" -> TSchema.int.opt, "y" -> TSchema.int.opt)))
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
          val json     = """[{"a": 1, "b": true}, {"a": 1, "b": true}, {"a": 2, "b": false}]"""
          val expected = TSchema.arr(TSchema.obj("a" -> TSchema.Int, "b" -> TSchema.Boolean))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("nullables to TSchema") {
          val json     = """[{"a": 1}, {"a": null}]"""
          val expected = TSchema.arr(TSchema.obj("a" -> (TSchema.Int.opt)))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("nullables with multiple keys to TSchema") {
          val json     = """[{"a": 1, "b": null}, {"a": null, "b": 1}]"""
          val expected = TSchema.arr(TSchema.obj("a" -> TSchema.int.opt, "b" -> TSchema.int.opt))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        suite("dictionary")(test("dictionary to TSchema") {
          val json     = Json.Obj("a" -> Json.Num(1), "b" -> Json.Num(1), "c" -> Json.Num(1))
          val expected = TSchema.dict(TSchema.Int)
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        }),
      ),
    )
}
