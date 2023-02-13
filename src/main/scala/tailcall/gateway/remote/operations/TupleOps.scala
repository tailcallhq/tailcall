package tailcall.gateway.remote.operations

import tailcall.gateway.remote.DynamicEval.TupleOperations
import tailcall.gateway.remote.{DynamicEval, Remote}

trait TupleOps {
  implicit final class RemoteTupleOps[A, B](val self: Remote[(A, B)]) {
    def _1: Remote[A] =
      Remote.unsafe.attempt(DynamicEval.TupleOperations(TupleOperations.GetIndex(self.compile, 0)))
    def _2: Remote[B] =
      Remote.unsafe.attempt(DynamicEval.TupleOperations(TupleOperations.GetIndex(self.compile, 1)))
  }
}
