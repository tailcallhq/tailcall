package tailcall.gateway.remote

import tailcall.gateway.remote.Remote.{EqualityTag, NumericTag}

import java.util.concurrent.atomic.AtomicInteger

sealed trait Remote[A] {
  self =>

  final def increment(implicit tag: NumericTag[A]) = self + Remote(tag.one)

  final def apply[A1, A2](a1: Remote[A1])(implicit ev: Remote[A] =:= Remote[A1 => A2]): Remote[A2] =
    Remote.Apply[A1, A2](ev(self).asInstanceOf[Remote.Closure.RemoteFunction[A1, A2]], a1)

  final def diverge[B](isTrue: Remote[B], isFalse: Remote[B])(implicit
    ev: Remote[A] =:= Remote[Boolean]
  ): Remote[B] = Remote.Diverge(ev(self), isTrue, isFalse)

  final def =:=(other: Remote[A])(implicit tag: EqualityTag[A]): Remote[Boolean] = Remote
    .EqualTo(self, other, tag)

  final def +(other: Remote[A])(implicit tag: NumericTag[A]): Remote[A] = Remote
    .Math(Remote.Math.Add(self, other), tag)

  final def -(other: Remote[A])(implicit tag: NumericTag[A]): Remote[A] = self + other.negate

  final def *(other: Remote[A])(implicit tag: NumericTag[A]): Remote[A] = Remote
    .Math(Remote.Math.Multiply(self, other), tag)

  final def /(other: Remote[A])(implicit tag: NumericTag[A]): Remote[A] = Remote
    .Math(Remote.Math.Divide(self, other), tag)

  final def %(other: Remote[A])(implicit tag: NumericTag[A]): Remote[A] = Remote
    .Math[A](Remote.Math.Modulo(self, other), tag)

  final def negate(implicit tag: NumericTag[A]): Remote[A] = Remote
    .Math(Remote.Math.Negate(self), tag)

  final def &&(other: Remote[A])(implicit ev: Remote[A] =:= Remote[Boolean]): Remote[Boolean] =
    Remote.Logical(Remote.Logical.And(self, other))

  final def ||(other: Remote[A])(implicit ev: Remote[A] =:= Remote[Boolean]): Remote[Boolean] =
    Remote.Logical(Remote.Logical.Or(self, other))

  final def unary_!(implicit ev: Remote[A] =:= Remote[Boolean]): Remote[Boolean] = Remote
    .Logical(Remote.Logical.Not(self))

  final def ++[B](
    other: Remote[IndexedSeq[B]]
  )(implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[IndexedSeq[B]] = Remote
    .IndexSeqOperations(Remote.IndexSeqOperations.Concat(ev(self), other))

  final def reverse[B](implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[IndexedSeq[B]] =
    Remote.IndexSeqOperations(Remote.IndexSeqOperations.Reverse(ev(self)))

  final def filter[B](
    f: Remote[B] => Remote[Boolean]
  )(implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[IndexedSeq[B]] = Remote
    .IndexSeqOperations(Remote.IndexSeqOperations.Filter(ev(self), Remote.Closure.make(f)))

  final def flatMap[B, C](
    f: Remote[B] => Remote[IndexedSeq[C]]
  )(implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[IndexedSeq[C]] = Remote
    .IndexSeqOperations(Remote.IndexSeqOperations.FlatMap(ev(self), f))

  final def map[B, C](f: Remote[B] => Remote[C])(implicit
    ev: Remote[A] =:= Remote[IndexedSeq[B]]
  ): Remote[IndexedSeq[C]] = Remote.IndexSeqOperations(Remote.IndexSeqOperations.Map(ev(self), f))

  final def length[B](implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[Int] = Remote
    .IndexSeqOperations(Remote.IndexSeqOperations.Length(ev(self)))

  final def indexOf[B](other: Remote[B])(implicit
    ev: Remote[A] =:= Remote[IndexedSeq[B]]
  ): Remote[Int] = Remote.IndexSeqOperations(Remote.IndexSeqOperations.IndexOf(ev(self), other))
}

object Remote extends RemoteTags with RemoteCtors {
  final case class Literal[A](value: A) extends Remote[A]

  final case class Diverge[A](cond: Remote[Boolean], isTrue: Remote[A], isFalse: Remote[A])
      extends Remote[A]

  final case class EqualTo[A](left: Remote[A], right: Remote[A], tag: EqualityTag[A])
      extends Remote[Boolean]

  final case class Math[A](operation: Math.Operation[A], tag: NumericTag[A]) extends Remote[A]

  object Math {
    sealed trait Operation[A]

    final case class Add[A](left: Remote[A], right: Remote[A]) extends Operation[A]

    final case class Negate[A](value: Remote[A]) extends Operation[A]

    final case class Multiply[A](left: Remote[A], right: Remote[A]) extends Operation[A]

    final case class Divide[A](left: Remote[A], right: Remote[A]) extends Operation[A]

    final case class Modulo[A](left: Remote[A], right: Remote[A]) extends Operation[A]
  }

  final case class Logical(operation: Logical.Operation) extends Remote[Boolean]
  object Logical {
    sealed trait Operation

    final case class And(left: Remote[Boolean], right: Remote[Boolean]) extends Operation

    final case class Or(left: Remote[Boolean], right: Remote[Boolean]) extends Operation

    final case class Not(value: Remote[Boolean]) extends Operation
  }

  final case class IndexSeqOperations[A](operation: IndexSeqOperations.Operation[A])
      extends Remote[A]

  object IndexSeqOperations {
    sealed trait Operation[A]

    final case class Concat[A](left: Remote[IndexedSeq[A]], right: Remote[IndexedSeq[A]])
        extends Operation[IndexedSeq[A]]

    final case class Reverse[A](seq: Remote[IndexedSeq[A]]) extends Operation[IndexedSeq[A]]

    final case class Filter[A](seq: Remote[IndexedSeq[A]], condition: Remote[A => Boolean])
        extends Operation[IndexedSeq[A]]

    final case class FlatMap[A, B](
      seq: Remote[IndexedSeq[A]],
      operation: Remote[A] => Remote[IndexedSeq[B]]
    ) extends Operation[IndexedSeq[B]]

    final case class Map[A, B](seq: Remote[IndexedSeq[A]], operation: Remote[A] => Remote[B])
        extends Operation[IndexedSeq[B]]

    final case class Length[A](seq: Remote[IndexedSeq[A]]) extends Operation[Int]

    final case class IndexOf[A](seq: Remote[IndexedSeq[A]], element: Remote[A])
        extends Operation[Int]
  }

  final case class Apply[A1, A2](f: Closure.RemoteFunction[A1, A2], arg: Remote[A1])
      extends Remote[A2]

  sealed trait Closure[A] extends Remote[A]
  object Closure {
    val counter = new AtomicInteger(0)
    final case class Ref[A] private (id: Int)                        extends Closure[A]
    final case class RemoteFunction[A, B](ref: Ref[A], f: Remote[B]) extends Closure[A => B]

    def make[A, B](ab: Remote[A] => Remote[B]): Remote[A => B] = {
      val id = Ref[A](counter.incrementAndGet())
      Closure.RemoteFunction(id, ab(id))
    }
  }
}
