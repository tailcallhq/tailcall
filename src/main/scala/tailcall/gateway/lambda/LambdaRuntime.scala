package tailcall.gateway.lambda

import zio._

trait LambdaRuntime {
  def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B]
}

object LambdaRuntime {
  import DynamicEval._

  final class Live(ctx: EvaluationContext) extends LambdaRuntime {
    def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B] =
      evaluate(lambda.compile(CompilationContext.initial)).asInstanceOf[LExit[Any, Throwable, A, B]]

    def evaluateAs[A](eval: DynamicEval): LExit[Any, Throwable, Any, A] =
      evaluate(eval).flatMap(a => LExit.attempt(a.asInstanceOf[A]))

    def evaluate(plan: DynamicEval): LExit[Any, Throwable, Any, Any] = {
      plan match {
        case Literal(value, ctor)              => ctor.schema.fromDynamic(value) match {
            case Left(cause)  => LExit.fail(EvaluationError.TypeError(value, cause, ctor.schema))
            case Right(value) => LExit.succeed(value)
          }
        case Literal(value, ctor)              => value.toTypedValue(ctor.schema)
            .fold(cause => LExit.fail(EvaluationError.TypeError(value, cause, ctor.schema)), LExit.succeed)
        case EqualTo(left, right, tag)         => for {
            leftValue  <- evaluate(left)
            rightValue <- evaluate(right)
          } yield tag.equal(leftValue, rightValue)
        case Math(operation, tag)              => operation match {
            case Math.Binary(operation, left, right) =>
              for {
                leftValue  <- evaluate(left)
                rightValue <- evaluate(right)
              } yield operation match {
                case Math.Binary.Add              => tag.add(leftValue, rightValue)
                case Math.Binary.Multiply         => tag.multiply(leftValue, rightValue)
                case Math.Binary.Divide           => tag.divide(leftValue, rightValue)
                case Math.Binary.Modulo           => tag.modulo(leftValue, rightValue)
                case Math.Binary.GreaterThan      => tag.greaterThan(leftValue, rightValue)
                case Math.Binary.GreaterThanEqual => tag.greaterThanEqual(leftValue, rightValue)
              }
            case Math.Unary(operation, value)        =>
              for { value <- evaluate(value) } yield operation match { case Math.Unary.Negate => tag.negate(value) }
          }
        case Logical(operation)                => operation match {
            case Logical.Binary(operation, left, right) =>
              for {
                leftValue  <- evaluateAs[Boolean](left)
                rightValue <- evaluateAs[Boolean](right)
              } yield operation match {
                case Logical.Binary.And => leftValue && rightValue
                case Logical.Binary.Or  => leftValue || rightValue
              }
            case Logical.Unary(value, operation)        => evaluateAs[Boolean](value).flatMap { a =>
                operation match {
                  case Logical.Unary.Not                      => LExit.succeed(!a)
                  case Logical.Unary.Diverge(isTrue, isFalse) => if (a) evaluate(isTrue) else evaluate(isFalse)
                }
              }
          }
        case Identity                          => LExit.input
        case Pipe(left, right)                 => evaluate(left) >>> evaluate(right)
        case FunctionDef(binding, body, input) => for {
            i <- evaluate(input)
            _ <- LExit.fromZIO(ctx.set(binding, i))
            r <- evaluate(body)
            _ <- LExit.fromZIO(ctx.drop(binding))
          } yield r
        case Lookup(binding)                   => LExit.fromZIO(ctx.get(binding))

        case Immediate(eval0) => for {
            eval1 <- evaluate(eval0)
            eval2 <- evaluate(eval1.asInstanceOf[DynamicEval])
          } yield eval2
        case Defer(value)     => LExit.succeed(value)
      }
    }
  }

  def live: ZLayer[EvaluationContext, Nothing, LambdaRuntime] =
    ZLayer.fromZIO(ZIO.service[EvaluationContext].map(new Live(_)))

  def evaluate[A, B](ab: A ~> B): LExit[LambdaRuntime, Throwable, A, B] =
    LExit.fromZIO(ZIO.service[LambdaRuntime]).flatMap(_.evaluate(ab))
}
