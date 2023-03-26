package tailcall.runtime

import tailcall.runtime.ast.TSchema
import tailcall.runtime.transcoder.Transcoder
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertZIO}

object TranscoderSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("TranscoderSpec")(suite("json to TSchema")(
      test("object to tSchema") {
        val json    = """{"a": 1, "b": "2", "c": true, "d": null, "e": [1, 2, 3]}"""
        val tSchema = TSchema.obj(
          "a" -> TSchema.Int,
          "b" -> TSchema.String,
          "c" -> TSchema.Boolean,
          "d" -> TSchema.NULL,
          "e" -> TSchema.arr(TSchema.Int),
        )
        assertZIO(Transcoder.toTSchema(json).toZIO)(equalTo(tSchema))
      },
      test("array tSchema") {
        val json    = """[{"a": 1, "b": true}, {"a": 1, "b": true}, {"a": 2, "b": false}]"""
        val tSchema = TSchema.arr(TSchema.obj("a" -> TSchema.Int, "b" -> TSchema.Boolean))
        assertZIO(Transcoder.toTSchema(json).toZIO)(equalTo(tSchema))
      },
      test("nullables to tSchema") {
        val json    = """[{"a": 1}, {"a": null}]"""
        val tSchema = TSchema.arr(TSchema.obj("a" -> (TSchema.Int | TSchema.NULL)))
        assertZIO(Transcoder.toTSchema(json).toZIO)(equalTo(tSchema))
      },
    ))
}
