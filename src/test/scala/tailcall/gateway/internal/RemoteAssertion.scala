package tailcall.gateway.internal

import tailcall.gateway.remote.{Remote, UnsafeEvaluator}
import zio.internal.stacktracer.SourceLocation
import zio.test.{Assertion, TestResult, assertZIO}
import zio.{Task, Trace, ZIO}

trait RemoteAssertion {
  def assertRemote[A](remote: Remote[A])(
    assertion: Assertion[A]
  )(implicit trace: Trace, sourceLocation: SourceLocation): Task[TestResult] = {
    val result = ZIO.attempt(UnsafeEvaluator.make().evaluateAs[A](remote.compile))
    assertZIO(result)(assertion)
  }
}
