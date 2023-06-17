package tailcall.runtime

import tailcall.runtime.service.JSExecutor
import tailcall.test.TailcallSpec
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, assertZIO}

object JSExecutorSpec extends TailcallSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JSExecutor")(test("basic evaluation") {
      val program = JSExecutor.execute("function (a) { return a + 1}", 100)
      assertZIO(program)(equalTo(101))
    }).provide(JSExecutor.live, Scope.default)
}
