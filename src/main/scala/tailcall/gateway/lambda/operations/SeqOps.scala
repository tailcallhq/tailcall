package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.SeqOperations
import tailcall.gateway.lambda.{Constructor, Lambda, Remote}

trait SeqOps {
  implicit final class RemoteSeqOps[A](val self: Remote[Seq[A]]) {
    def ++(other: Remote[Seq[A]]): Remote[Seq[A]] =
      Lambda.unsafe.attempt(ctx => SeqOperations(SeqOperations.Concat(self.compile(ctx), other.compile(ctx))))

    def reverse: Remote[Seq[A]] = Lambda.unsafe.attempt(ctx => SeqOperations(SeqOperations.Reverse(self.compile(ctx))))

    def filter(f: Remote[A] => Remote[Boolean]): Remote[Seq[A]] =
      Lambda.unsafe
        .attempt(ctx => SeqOperations(SeqOperations.Filter(self.compile(ctx), Lambda.fromFunction(f).compile(ctx))))

    def find(f: Remote[A] => Remote[Boolean]): Remote[Option[A]] = filter(f).head

    def flatMap[B](f: Remote[A] => Remote[Seq[B]]): Remote[Seq[B]] =
      Lambda.unsafe
        .attempt(ctx => SeqOperations(SeqOperations.FlatMap(self.compile(ctx), Lambda.fromFunction(f).compile(ctx))))

    def map[B](f: Remote[A] => Remote[B])(implicit ctor: Constructor[B]): Remote[Seq[B]] =
      self.flatMap(a => Lambda.fromSeq(Seq(f(a))))

    def length: Remote[Int] = Lambda.unsafe.attempt(ctx => SeqOperations(SeqOperations.Length(self.compile(ctx))))

    def indexOf(other: Remote[A]): Remote[Int] =
      Lambda.unsafe.attempt(ctx => SeqOperations(SeqOperations.IndexOf(self.compile(ctx), other.compile(ctx))))

    def take(n: Int): Remote[Seq[A]] = slice(0, n)

    def slice(from: Int, until: Int): Remote[Seq[A]] =
      Lambda.unsafe.attempt(ctx => SeqOperations(SeqOperations.Slice(self.compile(ctx), from, until)))

    def head: Remote[Option[A]] = Lambda.unsafe.attempt(ctx => SeqOperations(SeqOperations.Head(self.compile(ctx))))

    def groupBy[B](f: Remote[A] => Remote[B]): Remote[Map[B, Seq[A]]] =
      Lambda.unsafe
        .attempt(ctx => SeqOperations(SeqOperations.GroupBy(self.compile(ctx), Lambda.fromFunction(f).compile(ctx))))

    def batch[B1, C1](
      to: Remote[Seq[B1]] => Remote[Seq[C1]],
      ab: Remote[A] => Remote[B1],
      ba: Remote[B1] => Remote[A],
      cb: Remote[C1] => Remote[B1]
    )(implicit ctorB: Constructor[B1], ctorC: Constructor[C1], ctorF: Constructor[(A, Option[C1])]) = {

      val v = self.map(ab(_))
      v.map(i =>
        Lambda.fromTuple(
          ba(i),
          to(v).map(c => Lambda.fromTuple((cb(c), c))).groupBy(_._1).get(i)
            .flatMap(x => x.map(_._2).head) // Todo: Add flatten in Option
        )
      )
    }

  }
}
