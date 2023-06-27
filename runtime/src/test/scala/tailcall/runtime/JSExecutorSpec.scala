package tailcall.runtime

import tailcall.runtime.service.JSExecutor
import tailcall.test.TailcallSpec
import zio.test.Assertion.{equalTo, isLeft}
import zio.test.{Spec, TestEnvironment, assertZIO}
import zio.{Scope, durationInt}

object JSExecutorSpec extends TailcallSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JSExecutor")(
      test("basic evaluation") {
        val program = JSExecutor.execute("function (a) { return a + 1}", 100)
        assertZIO(program)(equalTo(101))
      },
      test("long running") {
        val program = JSExecutor.execute("function (a) { while (true) {a = a + 1}  return a}", 100)
          .mapError(_.getMessage).either
        assertZIO(program)(isLeft(equalTo("Execution got interrupted")))
      },
    ).provide(JSExecutor.live(5 millis))
}
