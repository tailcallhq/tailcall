package tailcall.gateway.internal

import tailcall.gateway.remote.{Remote, RemoteEval}
import zio.internal.stacktracer.SourceLocation
import zio.test.{Assertion, TestResult, assertZIO}
import zio.{Trace, ZIO}

trait RemoteAssertion {
  def assertRemote[A](remote: Remote[A])(
    assertion: Assertion[A]
  )(implicit trace: Trace, sourceLocation: SourceLocation): ZIO[Any, Nothing, TestResult] = {
    val evaluator = RemoteEval.make
    assertZIO(evaluator.eval(remote))(assertion)
  }
}
