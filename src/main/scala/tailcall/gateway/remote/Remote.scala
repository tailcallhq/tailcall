package tailcall.gateway.remote

import tailcall.gateway.remote.operations._
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

object Remote extends RemoteCtors with StringOps with SeqOps with BooleanOps {

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
