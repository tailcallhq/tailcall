package tailcall.runtime

import tailcall.runtime.ast.TSchema
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test._

object TSchemaSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("TSchemaSpec")(suite("merge")(
      test("removes duplicates") {
        val schema = TSchema.int unify TSchema.int
        assert(schema)(equalTo(TSchema.int))
      },
      test("unifies different types") {
        val schema = TSchema.int unify TSchema.string
        assert(schema)(equalTo(TSchema.int | TSchema.string))
      },
      test("merges fields of object types") {
        val schema = TSchema.obj("a" -> TSchema.int) unify TSchema.obj("b" -> TSchema.string)
        assert(schema)(equalTo(
          TSchema.obj("a" -> (TSchema.int | TSchema.NULL), "b" -> (TSchema.string | TSchema.NULL))
        ))
      },
    ))
}
