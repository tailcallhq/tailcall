package tailcall.gateway.remote.operations

import tailcall.gateway.remote.Remote
import tailcall.gateway.remote.DynamicEval

trait FunctionOps {
  implicit final class FunctionOps[A, B](private val self: Remote[A => B]) {
    def apply(a1: Remote[A]): Remote[B] = Remote.unsafe
      .attempt(DynamicEval.call(self.compileAsFunction, a1.compile))

    def compileAsFunction: DynamicEval.EvalFunction = self.compile
      .asInstanceOf[DynamicEval.EvalFunction]
  }
}
