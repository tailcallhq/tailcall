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
    attempt(DynamicEval.Apply(ev(self).asInstanceOf[DynamicEval.EvalFunction], a1.compile))

  final def diverge[B](isTrue: Remote[B], isFalse: Remote[B])(implicit
    ev: A =:= Boolean
  ): Remote[B] = attempt(DynamicEval.diverge(self.compile, isTrue.compile, isFalse.compile))

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

  final def &&(other: Remote[A])(implicit ev: Remote[A] =:= Remote[Boolean]): Remote[Boolean] =
    attempt(DynamicEval.and(self.compile, other.compile))

  final def ||(other: Remote[A])(implicit ev: Remote[A] =:= Remote[Boolean]): Remote[Boolean] =
    attempt(DynamicEval.or(self.compile, other.compile))

  final def unary_!(implicit ev: Remote[A] =:= Remote[Boolean]): Remote[Boolean] =
    attempt(DynamicEval.not(self.compile))

  final def reverse[B](implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[IndexedSeq[B]] =
    attempt(
      DynamicEval.IndexSeqOperations(DynamicEval.IndexSeqOperations.Reverse(ev(self).compile))
    )

  final def filter[B](
    f: Remote[B] => Remote[Boolean]
  )(implicit ev: Remote[A] =:= Remote[IndexedSeq[B]], schema: Schema[B]): Remote[IndexedSeq[B]] =
    ???

  final def flatMap[B, C](f: Remote[B] => Remote[IndexedSeq[C]])(implicit
    ev: Remote[A] =:= Remote[IndexedSeq[B]]
  ): Remote[IndexedSeq[C]] = ???

  final def map[B, C](f: Remote[B] => Remote[C])(implicit
    ev: Remote[A] =:= Remote[IndexedSeq[B]]
  ): Remote[IndexedSeq[C]] = ???

  final def length[B](implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[Int] =
    attempt(DynamicEval.IndexSeqOperations(DynamicEval.IndexSeqOperations.Length(ev(self).compile)))

  final def indexOf[B](
    other: Remote[B]
  )(implicit ev: Remote[A] =:= Remote[IndexedSeq[B]]): Remote[Int] = attempt(
    DynamicEval
      .IndexSeqOperations(DynamicEval.IndexSeqOperations.IndexOf(ev(self).compile, other.compile))
  )
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
      .attempt(DynamicEval.IndexSeqOperations(
        DynamicEval.IndexSeqOperations.Concat(self.compile, other.compile)
      ))
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
