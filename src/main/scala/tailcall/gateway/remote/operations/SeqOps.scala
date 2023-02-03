package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait SeqOps {
  implicit final class RemoteSeqOps[A](val self: Remote[IndexedSeq[A]]) {
    def ++(other: Remote[IndexedSeq[A]]): Remote[IndexedSeq[A]] = Remote.unsafe
      .attempt(DynamicEval.concat(self.compile, other.compile))

    final def reverse: Remote[IndexedSeq[A]] = Remote.unsafe
      .attempt(DynamicEval.reverse(self.compile))

    final def filter(f: Remote[A] => Remote[Boolean]): Remote[IndexedSeq[A]] = Remote.unsafe
      .attempt(DynamicEval.filter(
        self.compile,
        Remote.fromFunction(f).compile.asInstanceOf[DynamicEval.EvalFunction]
      ))

    final def flatMap[B](f: Remote[A] => Remote[IndexedSeq[B]]): Remote[IndexedSeq[B]] = Remote
      .unsafe.attempt(DynamicEval.flatMap(
        self.compile,
        Remote.fromFunction(f).compile.asInstanceOf[DynamicEval.EvalFunction]
      ))

    final def map[B](f: Remote[A] => Remote[B]): Remote[IndexedSeq[B]] = self
      .flatMap(a => Remote.seq(Seq(f(a))))

    final def length: Remote[Int] = Remote.unsafe.attempt(DynamicEval.length(self.compile))

    final def indexOf(other: Remote[A]): Remote[Int] = Remote.unsafe
      .attempt(DynamicEval.indexOf(self.compile, other.compile))
  }
}
