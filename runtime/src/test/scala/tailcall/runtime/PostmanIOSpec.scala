package tailcall.runtime

import tailcall.runtime.service.PostmanIO
import zio.test.Assertion.anything
import zio.test._
import zio.{Scope, ZIO}

object PostmanIOSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("PostmanIOSpec")(test("data model") {
      checkAll(Gen.fromIterable(1 to 4)) { i =>
        val file = PostmanIO.read(getClass.getResource(s"Postman_${i}.json"))
          .tap(file => ZIO.succeed(pprint.pprintln(file)))
        assertZIO(file)(anything)
      }
    }).provide(PostmanIO.default)
}
