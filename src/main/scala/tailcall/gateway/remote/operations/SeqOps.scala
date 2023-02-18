package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait SeqOps {
  implicit final class RemoteSeqOps[A](val self: Remote[Seq[A]]) {
    def ++(other: Remote[Seq[A]]): Remote[Seq[A]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.concat(self.compile(ctx), other.compile(ctx))
        )

    final def reverse: Remote[Seq[A]] =
      Remote.unsafe.attempt(ctx => DynamicEval.reverse(self.compile(ctx)))

    final def filter(f: Remote[A] => Remote[Boolean]): Remote[Seq[A]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.filter(
            self.compile(ctx),
            Remote.fromFunction(f).compileAsFunction(ctx)
          )
        )

    def find(f: Remote[A] => Remote[Boolean]): Remote[Option[A]] =
      filter(f).head

    final def flatMap[B](f: Remote[A] => Remote[Seq[B]]): Remote[Seq[B]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.flatMap(
            self.compile(ctx),
            Remote.fromFunction(f).compileAsFunction(ctx)
          )
        )

    final def map[B](f: Remote[A] => Remote[B]): Remote[Seq[B]] =
      self.flatMap(a => Remote.fromSeq(Seq(f(a))))

    final def length: Remote[Int] =
      Remote.unsafe.attempt(ctx => DynamicEval.length(self.compile(ctx)))

    final def indexOf(other: Remote[A]): Remote[Int] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.indexOf(self.compile(ctx), other.compile(ctx))
        )

    final def take(n: Int): Remote[Seq[A]] = slice(0, n)

    final def slice(from: Int, until: Int): Remote[Seq[A]] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.slice(self.compile(ctx), from, until))

    final def head: Remote[Option[A]] =
      Remote.unsafe.attempt(ctx => DynamicEval.head(self.compile(ctx)))

    final def groupBy[B](f: Remote[A] => Remote[B]): Remote[Map[B, Seq[A]]] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.groupBy(
            self.compile(ctx),
            Remote.fromFunction(f).compileAsFunction(ctx)
          )
        )
  }
}
