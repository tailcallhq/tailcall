package tailcall.gateway.lambda

import tailcall.gateway.remote._
import zio._

trait ExecutionRuntime {
  def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B]
}

object ExecutionRuntime {
  import ExecutionPlan._

  final class Live(ctx: EvaluationContext) extends ExecutionRuntime {
    def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B] =
      evaluate(lambda.compile(CompilationContext.initial))
        .asInstanceOf[LExit[Any, Throwable, A, B]]

    def evaluate(plan: ExecutionPlan): LExit[Any, Throwable, Any, Any] =
      plan match {
        case Constant(value, ctor) => ctor.schema.fromDynamic(value) match {
            case Left(cause)  =>
              LExit.fail(EvaluationError.TypeError(value, cause, ctor.schema))
            case Right(value) => LExit.succeed(value)
          }

        case Pipe(left, right) => evaluate(left) >>> evaluate(right)

        case Add(left, right) => for {
            left  <- evaluate(left)
            right <- evaluate(right)
          } yield left.asInstanceOf[Int] + right.asInstanceOf[Int]

        case Lookup(key) => LExit.fromZIO(
            ctx.get(key).mapError(_ => EvaluationError.BindingNotFound(key))
          )

        case FunctionDefinition(key, body) => for {
            any <- LExit.input[Any]
            _   <- LExit.fromZIO(ctx.set(key, any))
            res <- evaluate(body)
            _   <- LExit.fromZIO(ctx.drop(key))
          } yield res

        case Flatten(eval) => for {
            inner <- evaluate(eval)
            outer <- evaluate(
              inner
                .asInstanceOf[Lambda[_, _]]
                .compile(CompilationContext.initial)
            )
          } yield outer
      }
  }

  def live: ZLayer[EvaluationContext, Nothing, ExecutionRuntime] =
    ZLayer.fromZIO(ZIO.service[EvaluationContext].map(new Live(_)))

  def evaluate[A, B](ab: A ~> B): LExit[ExecutionRuntime, Throwable, A, B] =
    LExit.fromZIO(ZIO.service[ExecutionRuntime]).flatMap(_.evaluate(ab))
}
