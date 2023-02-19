package tailcall.gateway.remote

import zio.ZIO
import zio.schema.Schema

sealed trait Lambda[-A, +B] {
  self =>
  def compile(ctx: CompilationContext): DynamicEval

  final def evaluate: LExit[LambdaRuntime, Throwable, A, B] =
    LambdaRuntime.evaluate(self)

  final def evaluateWith(a: A): ZIO[LambdaRuntime, Throwable, B] =
    evaluate(a)

  final def >>>[B1 >: B, C](other: B1 ~> C): A ~> C =
    Lambda
      .unsafe
      .attempt[A, C](ctx =>
        DynamicEval.FunctionOperations(
          DynamicEval.FunctionOperations.Pipe(compile(ctx), other.compile(ctx))
        )
      )

  final def apply[A1 <: A](a: A1)(implicit ev: Constructor[A1]): Lazy[B] =
    Lambda(a) >>> self

  final def apply(a: Lazy[A]): Lazy[B] =
    a >>> self

}

object Lambda {
  object unsafe {
    def attempt[A, B](c: CompilationContext => DynamicEval): Lambda[A, B] =
      new Lambda[A, B] {
        override def compile(ctx: CompilationContext): DynamicEval = c(ctx)
      }
  }

  def apply[B](b: B)(implicit ctor: Constructor[B]): Any ~> B =
    Lambda
      .unsafe
      .attempt(_ =>
        DynamicEval.FunctionOperations(
          DynamicEval
            .FunctionOperations
            .Literal(ctor.schema.toDynamic(b), ctor.any)
        )
      )

  def fromFunction[A, B](f: Lazy[A] => Lazy[B]): A ~> B =
    Lambda
      .unsafe
      .attempt[A, B] { ctx =>
        val next = ctx.withNextLevel

        val key  = EvaluationContext.Key.fromContext(next)
        val body = f(
          Lambda
            .unsafe
            .attempt(_ =>
              DynamicEval
                .FunctionOperations(DynamicEval.FunctionOperations.Lookup(key))
            )
        ).compile(next)

        DynamicEval.FunctionOperations(
          DynamicEval.FunctionOperations.FunctionDefinition(key, body)
        )
      }

  def flatten[A, B](ab: A ~> (A ~> B)): A ~> B =
    Lambda
      .unsafe
      .attempt(ctx =>
        DynamicEval.FunctionOperations(
          DynamicEval.FunctionOperations.Flatten(ab.compile(ctx))
        )
      )

  implicit final class LambdaOps[A, B](val self: A ~> (A ~> B)) {
    def flatten = Lambda.flatten(self)
  }

  implicit val anySchema: Schema[Lambda[_, _]] = Schema[DynamicEval].transform(
    exe => Lambda.unsafe.attempt(_ => exe),
    _.compile(CompilationContext.initial)
  )

  implicit def schema[A, B]: Schema[A ~> B] =
    anySchema.asInstanceOf[Schema[A ~> B]]
}
