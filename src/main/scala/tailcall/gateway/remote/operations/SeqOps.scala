package tailcall.gateway.remote.operations

import tailcall.gateway.remote.DynamicEval.SeqOperations
import tailcall.gateway.remote.{Constructor, Lambda, Remote}

trait SeqOps {
  implicit final class RemoteSeqOps[A](val self: Remote[Seq[A]]) {
    def ++(other: Remote[Seq[A]]): Remote[Seq[A]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          SeqOperations(
            SeqOperations.Concat(self.compile(ctx), other.compile(ctx))
          )
        )

    def reverse: Remote[Seq[A]] =
      Lambda
        .unsafe
        .attempt(ctx => SeqOperations(SeqOperations.Reverse(self.compile(ctx))))

    def filter(f: Remote[A] => Remote[Boolean]): Remote[Seq[A]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          SeqOperations(
            SeqOperations
              .Filter(self.compile(ctx), Lambda.fromFunction(f).compile(ctx))
          )
        )

    def find(f: Remote[A] => Remote[Boolean]): Remote[Option[A]] =
      filter(f).head

    def flatMap[B](f: Remote[A] => Remote[Seq[B]]): Remote[Seq[B]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          SeqOperations(
            SeqOperations
              .FlatMap(self.compile(ctx), Lambda.fromFunction(f).compile(ctx))
          )
        )

    def map[B](f: Remote[A] => Remote[B])(implicit
      ctor: Constructor[B]
    ): Remote[Seq[B]] = self.flatMap(a => Lambda.fromSeq(Seq(f(a))))

    def length: Remote[Int] =
      Lambda
        .unsafe
        .attempt(ctx => SeqOperations(SeqOperations.Length(self.compile(ctx))))

    def indexOf(other: Remote[A]): Remote[Int] =
      Lambda
        .unsafe
        .attempt(ctx =>
          SeqOperations(
            SeqOperations.IndexOf(self.compile(ctx), other.compile(ctx))
          )
        )

    def take(n: Int): Remote[Seq[A]] = slice(0, n)

    def slice(from: Int, until: Int): Remote[Seq[A]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          SeqOperations(SeqOperations.Slice(self.compile(ctx), from, until))
        )

    def head: Remote[Option[A]] =
      Lambda
        .unsafe
        .attempt(ctx => SeqOperations(SeqOperations.Head(self.compile(ctx))))

    def groupBy[B](f: Remote[A] => Remote[B]): Remote[Map[B, Seq[A]]] =
      Lambda
        .unsafe
        .attempt(ctx =>
          SeqOperations(
            SeqOperations
              .GroupBy(self.compile(ctx), Lambda.fromFunction(f).compile(ctx))
          )
        )
  }
}
