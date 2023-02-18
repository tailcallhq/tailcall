package tailcall.gateway.remote

import tailcall.gateway.remote.operations._
import zio.ZIO
import zio.schema.{DynamicValue, Schema}

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

  def compile(context: CompilationContext): DynamicEval

  final def =:=[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Equatable[A1]): Remote[Boolean] =
    attempt(ctx =>
      DynamicEval.equal(self.compile(ctx), other.compile(ctx), tag.any)
    )

  final def >[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[Boolean] =
    attempt(ctx =>
      DynamicEval.greaterThan(self.compile(ctx), other.compile(ctx), tag.any)
    )

  final def increment[A1 >: A](implicit
    tag: Numeric[A1],
    ctor: Constructor[A1]
  ) = self + Remote(tag.one)

  final def +[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      DynamicEval.add(self.compile(ctx), other.compile(ctx), tag.any)
    )

  final def -[A1 >: A](other: Remote[A1])(implicit
    tag: Numeric[A1]
  ): Remote[A1] = self + other.negate

  final def *[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      DynamicEval.multiply(self.compile(ctx), other.compile(ctx), tag.any)
    )

  final def /[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      DynamicEval.divide(self.compile(ctx), other.compile(ctx), tag.any)
    )

  final def %[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      DynamicEval.modulo(self.compile(ctx), other.compile(ctx), tag.any)
    )

  final def negate[A1 >: A](implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx => DynamicEval.negate(self.compile(ctx), tag.any))

  final def debug(message: String): Remote[A] =
    attempt(ctx => DynamicEval.debug(self.compile(ctx), message))

  final def toDynamicValue[A1 >: A](implicit
    ev: Schema[A1]
  ): Remote[DynamicValue] = ???

  final def flatten[B](implicit
    ev: Remote[A] <:< Remote[Remote[B]]
  ): Remote[B] = Remote.flatten(self)

  final def evaluate: ZIO[RemoteRuntime, Throwable, A] =
    RemoteRuntime.evaluate(self)
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
    with DynamicValueOps
    with TupleOps
    with MapOps {

  object unsafe {
    object attempt {
      def apply[A](eval: CompilationContext => DynamicEval): Remote[A] =
        new Remote[A] {
          override def compile(context: CompilationContext): DynamicEval =
            eval(context)
        }
    }
  }

  implicit val anySchema: Schema[Remote[_]] = Schema[DynamicEval].transform(
    eval => unsafe.attempt(_ => eval),
    _.compile(CompilationContext.initial)
  )

  implicit def schema[A]: Schema[Remote[A]] =
    anySchema.asInstanceOf[Schema[Remote[A]]]

  implicit def remoteFunctionSchema[A, B]: Schema[Remote[A] => Remote[B]] =
    Schema[Remote[A => B]].transform(_.toFunction, Remote.fromFunction)
}
