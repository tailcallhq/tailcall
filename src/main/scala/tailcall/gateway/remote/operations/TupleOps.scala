package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait TupleOps {
  implicit final class RemoteTupleOps[A, B](val self: Remote[(A, B)]) {
    def _1: Remote[A] = Remote.unsafe.attempt(DynamicEval.tupleIndex(self.compile, 0))
    def _2: Remote[B] = Remote.unsafe.attempt(DynamicEval.tupleIndex(self.compile, 1))
  }
}
