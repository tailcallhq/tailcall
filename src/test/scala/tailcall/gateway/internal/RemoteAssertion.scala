package tailcall.gateway.internal

import tailcall.gateway.remote.{DynamicEval, Remote}
import zio.internal.stacktracer.SourceLocation
import zio.test.{Assertion, TestResult, assertZIO}
import zio.{Task, Trace, ZIO}

trait RemoteAssertion {
  def assertRemote[A](remote: Remote[A])(
    assertion: Assertion[A]
  )(implicit trace: Trace, sourceLocation: SourceLocation): Task[TestResult] = {
    val result = ZIO.attempt(DynamicEval.Unsafe.evaluate(remote.compile).asInstanceOf[A])
    assertZIO(result)(assertion)
  }
}
