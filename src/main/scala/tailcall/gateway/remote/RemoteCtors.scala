package tailcall.gateway.remote

import zio.Chunk
import zio.schema.Schema

trait RemoteCtors {
  def apply[A](a: A)(implicit schema: Schema[A]): Remote[A] = Remote.unsafe
    .attempt(DynamicEval.Literal(schema.toDynamic(a), schema.ast))

  def fromFunction[A, B](ab: Remote[A] => Remote[B]): Remote[A => B] = Remote.unsafe.attempt {
    val id = DynamicEval.binding
    DynamicEval.EvalFunction(id, ab(Remote.unsafe.attempt[A](id)).compile)
  }

  def seq[A](a: Seq[Remote[A]]): Remote[IndexedSeq[A]] = Remote.unsafe.attempt {
    val seq = a.map(_.compile)
    DynamicEval.IndexSeqOperations(DynamicEval.IndexSeqOperations.Sequence(Chunk.from(seq)))
  }
}
