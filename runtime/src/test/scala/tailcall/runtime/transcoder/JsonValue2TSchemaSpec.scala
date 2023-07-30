package tailcall.runtime.transcoder

import tailcall.TailcallSpec
import tailcall.runtime.model.TSchema
import zio.Scope
import zio.json.ast.Json
import zio.test.Assertion.{equalTo, isNone, isSome}
import zio.test._

object JsonValue2TSchemaSpec extends TailcallSpec with JsonValue2TSchema {
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    import tailcall.runtime.SchemaUnifier.unify
    suite("json to TSchema")(
      suite("unify")(
        test("removes duplicates") {
          val schema = unify(TSchema.num, TSchema.num)
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.num)))
        },
        test("objects") {
          val schema = unify(TSchema.obj("a" -> TSchema.num), TSchema.obj("b" -> TSchema.num))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.num.opt, "b" -> TSchema.num.opt))))
        },
        test("array") {
          val schema = unify(TSchema.obj("a" -> TSchema.num).arr, TSchema.obj("b" -> TSchema.num).arr)
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.num.opt, "b" -> TSchema.num.opt).arr)))
        },
        test("optional(a) b") {
          val schema = unify(TSchema.obj("a" -> TSchema.num.opt), TSchema.obj("a" -> TSchema.num))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.num.opt))))
        },
        test("a optional(b)") {
          val schema = unify(TSchema.obj("a" -> TSchema.num), TSchema.obj("a" -> TSchema.num.opt))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.num.opt))))
        },
        test("optional(a) optional(b)") {
          val schema = unify(TSchema.obj("a" -> TSchema.num.opt), TSchema.obj("a" -> TSchema.num.opt))
          assertZIO(schema.toZIO)(isSome(equalTo(TSchema.obj("a" -> TSchema.num.opt))))
        },
        test("int and string") {
          val schema = unify(TSchema.num, TSchema.str)
          assertZIO(schema.toZIO)(isNone)
        },
        test("deeply nested") {
          val schema = unify(
            TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("x" -> TSchema.num))),
            TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("y" -> TSchema.num))),
          )
          assertZIO(schema.toZIO)(isSome(
            equalTo(TSchema.obj("a" -> TSchema.obj("b" -> TSchema.obj("x" -> TSchema.num.opt, "y" -> TSchema.num.opt))))
          ))
        },
      ),
      suite("json to TSchema")(
        test("object to tSchema") {
          val json     = """{"a": 1, "b": "2", "c": true, "d": null, "e": [1, 2, 3]}"""
          val expected = TSchema.obj(
            "a" -> TSchema.Num,
            "b" -> TSchema.Str,
            "c" -> TSchema.Bool,
            "d" -> TSchema.empty,
            "e" -> TSchema.arr(TSchema.Num),
          )
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("array tSchema") {
          val json     = Json.Arr(
            Json.Obj("a" -> Json.Num(1), "b" -> Json.Bool(true)),
            Json.Obj("a" -> Json.Num(1), "b" -> Json.Bool(true)),
            Json.Obj("a" -> Json.Num(2), "b" -> Json.Bool(false)),
          )
          val expected = TSchema.arr(TSchema.obj("a" -> TSchema.Num, "b" -> TSchema.Bool))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("nullables to TSchema") {
          val json     = Json.Arr(Json.Obj("a" -> Json.Num(1)), Json.Obj("a" -> Json.Null))
          val expected = TSchema.arr(TSchema.obj("a" -> (TSchema.Num.opt)))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("nullables with multiple keys to TSchema") {
          val json     = Json
            .Arr(Json.Obj("a" -> Json.Num(1), "b" -> Json.Null), Json.Obj("a" -> Json.Null, "b" -> Json.Num(1)))
          val expected = TSchema.arr(TSchema.obj("a" -> TSchema.num.opt, "b" -> TSchema.num.opt))
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
        test("numeric keys") {
          val json     = Json.Obj("1" -> Json.Num(1), "2" -> Json.Str("1"), "3" -> Json.Bool(true))
          val expected = TSchema.obj("_1" -> TSchema.Num, "_2" -> TSchema.Str, "_3" -> TSchema.Bool)
          assertZIO(toTSchema(json).toZIO)(equalTo(expected))
        },
      ),
    )
  }
}
