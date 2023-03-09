package tailcall.runtime

import tailcall.runtime.internal.DynamicValueUtil._
import zio.schema._
import zio.test.Assertion._
import zio.test._

object DynamicValueUtilSpec extends ZIOSpecDefault {
  final case class FAQ(question: String, answer: Int)

  object FAQ {
    implicit val schema: Schema[FAQ] = DeriveSchema.gen[FAQ]
  }

  override def spec =
    suite("DynamicValueUtilSpec")(test("asString") {
      assert(asString(DynamicValue("answer")))(isSome(equalTo("answer"))) &&
      assert(asString(DynamicValue(42)))(isSome(equalTo("42"))) &&
      assert(asString(DynamicValue(FAQ("meaning of life", 42))))(isNone)
    })
}
