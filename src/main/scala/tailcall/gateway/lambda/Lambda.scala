package tailcall.gateway.lambda

import tailcall.gateway.remote._
import zio.ZIO
import zio.schema.Schema

sealed trait Lambda[-A, +B] {
  self =>
  def compile(ctx: CompilationContext): ExecutionPlan

  final def evaluate: LExit[ExecutionRuntime, Throwable, A, B] =
    ExecutionRuntime.evaluate(self)

  final def evaluateWith(a: A): ZIO[ExecutionRuntime, Throwable, B] =
    evaluate(a)

  final def >>>[B1 >: B, C](other: B1 ~> C): A ~> C =
    Lambda
      .unsafe
      .attempt[A, C](ctx =>
        ExecutionPlan.Pipe(compile(ctx), other.compile(ctx))
      )

  final def apply[A1 <: A](a: A1)(implicit ev: Constructor[A1]): Lazy[B] =
    Lambda(a) >>> self

}

object Lambda {
  object unsafe {
    def attempt[A, B](c: CompilationContext => ExecutionPlan): Lambda[A, B] =
      new Lambda[A, B] {
        override def compile(ctx: CompilationContext): ExecutionPlan = c(ctx)
      }
  }

  def apply[B](b: B)(implicit ctor: Constructor[B]): Any ~> B =
    Lambda
      .unsafe
      .attempt(_ => ExecutionPlan.Constant(ctor.schema.toDynamic(b), ctor.any))

  def fromFunction[A, B](f: Lazy[A] => Lazy[B]): A ~> B =
    Lambda
      .unsafe
      .attempt[A, B] { ctx =>
        val next = ctx.withNextLevel

        val key  = EvaluationContext.Key.fromContext(next)
        val body = f(Lambda.unsafe.attempt(_ => ExecutionPlan.Lookup(key)))
          .compile(next)

        ExecutionPlan.FunctionDefinition(key, body)
      }

  def flatten[A, B](ab: A ~> (A ~> B)): A ~> B =
    Lambda.unsafe.attempt(ctx => ExecutionPlan.Flatten(ab.compile(ctx)))

  implicit final class LazyOps(val self: Lazy[Int]) {
    def +(other: Lazy[Int]): Lazy[Int] =
      Lambda
        .unsafe
        .attempt(ctx =>
          ExecutionPlan.Add(self.compile(ctx), other.compile(ctx))
        )
  }

  implicit final class LambdaOps[A, B](val self: A ~> (A ~> B)) {
    def flatten = Lambda.flatten(self)
  }

  implicit val anySchema: Schema[Lambda[_, _]] = Schema[ExecutionPlan]
    .transform(
      exe => Lambda.unsafe.attempt(_ => exe),
      _.compile(CompilationContext.initial)
    )

  implicit def schema[A, B]: Schema[A ~> B] =
    anySchema.asInstanceOf[Schema[A ~> B]]
}
