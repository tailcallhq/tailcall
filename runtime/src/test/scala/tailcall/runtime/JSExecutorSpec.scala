package tailcall.runtime

import tailcall.runtime.service.JSExecutor
import tailcall.test.TailcallSpec
import zio.Scope
import zio.test.Assertion.equalTo
import zio.test.{Spec, TestEnvironment, assertZIO}

object JSExecutorSpec extends TailcallSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("JSExecutor")(
      test("basic evaluation") {
        val program = JSExecutor.execute("function (a) { return a + 1}", 100)
        assertZIO(program)(equalTo(101))
      },
      test("long running") {
        val program = for {
          fib    <- JSExecutor.execute("function (a) { while(true) {a = a + 1}  return a}", 100).fork
          result <- fib.join
        } yield result
        assertZIO(program)(equalTo(101))
      },
    ).provide(JSExecutor.live)
}
