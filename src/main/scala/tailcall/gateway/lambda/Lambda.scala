package tailcall.gateway.lambda

import tailcall.gateway.lambda.DynamicEval.{EqualTo, Literal, Math}
import zio.schema.Schema

sealed trait Lambda[-A, +B] {
  self =>
  def compile(context: CompilationContext): DynamicEval
  final def evaluate: LExit[LambdaRuntime, Throwable, A, B] =
    LambdaRuntime.evaluate(self)
}

object Lambda {

  def apply[A, B](b: B)(implicit ctor: Constructor[B]): A ~> B =
    Lambda
      .unsafe
      .attempt(_ =>
        Literal(ctor.schema.toDynamic(b), ctor.asInstanceOf[Constructor[Any]])
      )

  def add[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
    Lambda
      .unsafe
      .attempt(ctx =>
        Math(
          Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Add),
          ev.any
        )
      )

  def divide[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
    Lambda
      .unsafe
      .attempt(ctx =>
        Math(
          Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Divide),
          ev.any
        )
      )

  def multiply[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
    Lambda
      .unsafe
      .attempt(ctx =>
        Math(
          Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Multiply),
          ev.any
        )
      )

  def gt[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> Boolean =
    Lambda
      .unsafe
      .attempt(ctx =>
        Math(
          Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.GreaterThan),
          ev.any
        )
      )

  def subtract[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
    add(a, negate(b))

  def negate[A, B](ab: A ~> B)(implicit ev: Numeric[B]): A ~> B =
    Lambda
      .unsafe
      .attempt(ctx =>
        Math(Math.Unary(ab.compile(ctx), Math.Unary.Negate), ev.any)
      )

  def modulo[A, B](a: A ~> B, b: A ~> B)(implicit ev: Numeric[B]): A ~> B =
    Lambda
      .unsafe
      .attempt(ctx =>
        Math(
          Math.Binary(a.compile(ctx), b.compile(ctx), Math.Binary.Modulo),
          ev.any
        )
      )

  def equalTo[A, B](a: A ~> B, b: A ~> B)(implicit
    ev: Equatable[B]
  ): A ~> Boolean =
    Lambda
      .unsafe
      .attempt(ctx => EqualTo(a.compile(ctx), b.compile(ctx), ev.any))

  object unsafe {
    object attempt {
      def apply[A, B](eval: CompilationContext => DynamicEval): Lambda[A, B] =
        new Lambda[A, B] {
          override def compile(context: CompilationContext): DynamicEval =
            eval(context)
        }
    }
  }

  implicit val anySchema: Schema[Lambda[_, _]] = Schema[DynamicEval].transform(
    eval => Lambda.unsafe.attempt(_ => eval),
    _.compile(CompilationContext.initial)
  )

  implicit def schema[A, B]: Schema[Lambda[A, B]] =
    anySchema.asInstanceOf[Schema[Lambda[A, B]]]
}
