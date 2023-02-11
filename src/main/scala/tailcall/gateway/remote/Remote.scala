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
sealed trait Remote[+A] {
  self =>

  import Remote.unsafe.attempt

  // TODO: wrap inside an unsafe
  def compile: DynamicEval

  final def =:=[A1 >: A](other: Remote[A1])(implicit tag: Equatable[A1]): Remote[Boolean] =
    attempt(DynamicEval.equal(self.compile, other.compile, tag.any))

  final def increment[A1 >: A](implicit tag: Numeric[A1], schema: Schema[A1]) =
    self + Remote(tag.one)

  final def +[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(DynamicEval.add(self.compile, other.compile, tag.any))

  final def -[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
    self + other.negate

  final def *[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(DynamicEval.multiply(self.compile, other.compile, tag.any))

  final def /[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(DynamicEval.divide(self.compile, other.compile, tag.any))

  final def %[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(DynamicEval.modulo(self.compile, other.compile, tag.any))

  final def negate[A1 >: A](implicit tag: Numeric[A1]): Remote[A1] =
    attempt(DynamicEval.negate(self.compile, tag.any))
}

object Remote
    extends RemoteCtors
    with StringOps
    with SeqOps
    with BooleanOps
    with EitherOps
    with FunctionOps
    with OptionOps
    with ContextOps
    with DynamicValueOps {

  object unsafe {
    object attempt {
      def apply[A](eval: => DynamicEval): Remote[A] =
        new Remote[A] {
          override def compile: DynamicEval = eval
        }
    }
  }

  implicit val anySchema: Schema[Remote[_]] = Schema[DynamicEval]
    .transform(unsafe.attempt(_), _.compile)

  implicit def schema[A]: Schema[Remote[A]] = anySchema.asInstanceOf[Schema[Remote[A]]]

  implicit def remoteFunctionSchema[A, B]: Schema[Remote[A] => Remote[B]] =
    Schema[Remote[A => B]].transform(_.toFunction, Remote.fromFunction(_))
}
