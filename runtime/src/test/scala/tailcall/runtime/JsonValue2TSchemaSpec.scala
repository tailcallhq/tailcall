package tailcall.runtime

import tailcall.runtime.ast.TSchema
import tailcall.runtime.transcoder.JsonValue2TSchema
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test._

object JsonValue2TSchemaSpec extends ZIOSpecDefault with JsonValue2TSchema {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("TSchemaSpec")(
      suite("unify")(test("removes duplicates") {
        val schema = unify(TSchema.int, TSchema.int)
        assertZIO(schema.toZIO)(equalTo(TSchema.int))
      }),
      suite("json to TSchema")(
        test("object to tSchema") {
          val json    = """{"a": 1, "b": "2", "c": true, "d": null, "e": [1, 2, 3]}"""
          val tSchema = TSchema.obj(
            "a" -> TSchema.Int,
            "b" -> TSchema.String,
            "c" -> TSchema.Boolean,
            "d" -> TSchema.empty,
            "e" -> TSchema.arr(TSchema.Int),
          )
          assertZIO(toTSchema(json).toZIO)(equalTo(tSchema))
        },
        test("array tSchema") {
          val json    = """[{"a": 1, "b": true}, {"a": 1, "b": true}, {"a": 2, "b": false}]"""
          val tSchema = TSchema.arr(TSchema.obj("a" -> TSchema.Int, "b" -> TSchema.Boolean))
          assertZIO(toTSchema(json).toZIO)(equalTo(tSchema))
        },
        test("nullables to TSchema") {
          val json    = """[{"a": 1}, {"a": null}]"""
          val tSchema = TSchema.arr(TSchema.obj("a" -> (TSchema.Int.opt)))
          assertZIO(toTSchema(json).toZIO)(equalTo(tSchema))
        },
      ),
    )
}
