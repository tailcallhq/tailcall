package tailcall.gateway.remote

import zio.schema.Schema

/**
 * Remote[A] Allows for any arbitrary computation that can
 * be serialized and when evaluated produces a result of
 * type A. This is the lowest level primitive that’s
 * extremely powerful. We use this inside the compiler to
 * convert the composition logic into some form of a Remote.
 */
sealed trait Remote[A] {
  self =>

  import Remote.unsafe.attempt
  def compile: DynamicEval

  final def apply[A1, A2](a1: Remote[A1])(implicit ev: Remote[A] =:= Remote[A1 => A2]): Remote[A2] =
    attempt(
      DynamicEval.functionCall(self.compile.asInstanceOf[DynamicEval.EvalFunction], a1.compile)
    )

  final def =:=(other: Remote[A])(implicit tag: Equatable[A]): Remote[Boolean] =
    attempt(DynamicEval.equal(self.compile, other.compile, tag.any))

  final def increment(implicit tag: Numeric[A], schema: Schema[A]) = self + Remote(tag.one)

  final def +(other: Remote[A])(implicit tag: Numeric[A]): Remote[A] =
    attempt(DynamicEval.add(self.compile, other.compile, tag.any))

  final def -(other: Remote[A])(implicit tag: Numeric[A]): Remote[A] = self + other.negate

  final def *(other: Remote[A])(implicit tag: Numeric[A]): Remote[A] =
    attempt(DynamicEval.multiply(self.compile, other.compile, tag.any))

  final def /(other: Remote[A])(implicit tag: Numeric[A]): Remote[A] =
    attempt(DynamicEval.divide(self.compile, other.compile, tag.any))

  final def %(other: Remote[A])(implicit tag: Numeric[A]): Remote[A] =
    attempt(DynamicEval.modulo(self.compile, other.compile, tag.any))

  final def negate(implicit tag: Numeric[A]): Remote[A] =
    attempt(DynamicEval.negate(self.compile, tag.any))

}

object Remote extends RemoteTags with RemoteCtors {
  implicit final class ComposeStringInterpolator(val sc: StringContext) extends AnyVal {
    def rs[A](args: (Remote[String])*): Remote[String] = {
      val strings             = sc.parts.iterator
      val seq                 = args.iterator
      var buf: Remote[String] = Remote(strings.next())
      while (strings.hasNext) buf = buf ++ seq.next() ++ Remote(strings.next())
      buf
    }
  }

  implicit final class RemoteStringOps(val self: Remote[String]) extends AnyVal {
    def ++(other: Remote[String]): Remote[String] = unsafe.attempt(
      DynamicEval.StringOperations(DynamicEval.StringOperations.Concat(self.compile, other.compile))
    )
  }

  implicit final class RemoteSeqOps[A](val self: Remote[IndexedSeq[A]]) extends AnyVal {
    def ++(other: Remote[IndexedSeq[A]]): Remote[IndexedSeq[A]] = Remote.unsafe
      .attempt(DynamicEval.concat(self.compile, other.compile))

    final def reverse: Remote[IndexedSeq[A]] = unsafe.attempt(DynamicEval.reverse(self.compile))

    final def filter(f: Remote[A] => Remote[Boolean]): Remote[IndexedSeq[A]] = unsafe.attempt(
      DynamicEval
        .filter(self.compile, Remote.fromFunction(f).compile.asInstanceOf[DynamicEval.EvalFunction])
    )

    final def flatMap[B](f: Remote[A] => Remote[IndexedSeq[B]]): Remote[IndexedSeq[B]] = unsafe
      .attempt(DynamicEval.flatMap(
        self.compile,
        Remote.fromFunction(f).compile.asInstanceOf[DynamicEval.EvalFunction]
      ))

    final def map[B](f: Remote[A] => Remote[B]): Remote[IndexedSeq[B]] = self
      .flatMap(a => Remote.seq(Seq(f(a))))

    final def length: Remote[Int] = unsafe.attempt(DynamicEval.length(self.compile))

    final def indexOf(other: Remote[A]): Remote[Int] = unsafe
      .attempt(DynamicEval.indexOf(self.compile, other.compile))
  }

  implicit final class RemoteBooleanOps(val self: Remote[Boolean]) extends AnyVal {
    def &&(other: Remote[Boolean]): Remote[Boolean] = unsafe
      .attempt(DynamicEval.and(self.compile, other.compile))

    def ||(other: Remote[Boolean]): Remote[Boolean] = unsafe
      .attempt(DynamicEval.or(self.compile, other.compile))

    def unary_! : Remote[Boolean] = unsafe.attempt(DynamicEval.not(self.compile))

    def diverge[A](isTrue: Remote[A], isFalse: Remote[A]): Remote[A] = unsafe
      .attempt(DynamicEval.diverge(self.compile, isTrue.compile, isFalse.compile))
  }

  object unsafe {
    object attempt {
      def apply[A](eval: => DynamicEval): Remote[A] = new Remote[A] {
        override def compile: DynamicEval = eval
      }
    }
  }

  implicit val anySchema: Schema[Remote[_]] = Schema[DynamicEval]
    .transform(unsafe.attempt(_), _.compile)

  implicit def schema[A]: Schema[Remote[A]] = anySchema.asInstanceOf[Schema[Remote[A]]]
}
