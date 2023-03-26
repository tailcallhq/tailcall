package tailcall.runtime

import tailcall.runtime.service.PostmanIO
import zio.Scope
import zio.test.Assertion.anything
import zio.test._

object PostmanIOSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("PostmanIOSpec")(test("data model") {
      checkAll(Gen.fromIterable(1 to 4)) { i =>
        val file = PostmanIO.read(getClass.getResource(s"Postman_${i}.json"))
        assertZIO(file)(anything)
      }
    }).provide(PostmanIO.default)
}
