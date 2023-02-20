package tailcall.gateway.lambda

import tailcall.gateway.lambda.DynamicEval.{EqualTo, Literal, Math}
import tailcall.gateway.lambda.operations._
import zio.ZIO
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

  def compile(context: CompilationContext): DynamicEval

  final def =:=[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Equatable[A1]): Remote[Boolean] =
    attempt(ctx => EqualTo(self.compile(ctx), other.compile(ctx), tag.any))

  final def >[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[Boolean] =
    attempt(ctx =>
      Math(
        Math.Binary(
          self.compile(ctx),
          other.compile(ctx),
          Math.Binary.GreaterThan
        ),
        tag.any
      )
    )

  final def increment[A1 >: A](implicit
    tag: Numeric[A1],
    ctor: Constructor[A1]
  ) = self + Remote(tag.one)

  final def +[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      Math(
        Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Add),
        tag.any
      )
    )

  final def -[A1 >: A](other: Remote[A1])(implicit
    tag: Numeric[A1]
  ): Remote[A1] = self + other.negate

  final def *[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      Math(
        Math
          .Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Multiply),
        tag.any
      )
    )

  final def /[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      Math(
        Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Divide),
        tag.any
      )
    )

  final def %[A1 >: A](
    other: Remote[A1]
  )(implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      Math(
        Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Modulo),
        tag.any
      )
    )

  final def negate[A1 >: A](implicit tag: Numeric[A1]): Remote[A1] =
    attempt(ctx =>
      Math(Math.Unary(self.compile(ctx), Math.Unary.Negate), tag.any)
    )

  final def evaluate: ZIO[LambdaRuntime, Throwable, A] =
    LambdaRuntime.evaluate(self)
}

object Remote extends BooleanOps {

  object unsafe {
    object attempt {
      def apply[A](eval: CompilationContext => DynamicEval): Remote[A] =
        new Remote[A] {
          override def compile(context: CompilationContext): DynamicEval =
            eval(context)
        }
    }
  }

  def apply[A](a: A)(implicit ctor: Constructor[A]): Remote[A] =
    Remote
      .unsafe
      .attempt(_ =>
        Literal(ctor.schema.toDynamic(a), ctor.asInstanceOf[Constructor[Any]])
      )

  implicit val anySchema: Schema[Remote[_]] = Schema[DynamicEval].transform(
    eval => unsafe.attempt(_ => eval),
    _.compile(CompilationContext.initial)
  )

  implicit def schema[A]: Schema[Remote[A]] =
    anySchema.asInstanceOf[Schema[Remote[A]]]
}
