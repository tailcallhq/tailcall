package tailcall.gateway.lambda

import zio.{Task, ZIO, ZLayer}

trait RemoteRuntime {
  final def evaluateAs[A](eval: DynamicEval): Task[A] =
    evaluate(eval).flatMap(any => ZIO.attempt(any.asInstanceOf[A]))

  final def evaluate[A](remote: Remote[A]): Task[A] =
    evaluateAs[A](remote.compile(CompilationContext.initial))

  def evaluate(eval: DynamicEval): Task[Any]
}

object RemoteRuntime {
  import DynamicEval._
  final class Default(val context: EvaluationContext) extends RemoteRuntime {
    def evaluate(eval: DynamicEval): Task[Any] =
      eval match {
        case Literal(value, ctor)      => ZIO
            .fromEither(value.toTypedValue(ctor.schema))
            .mapError(cause =>
              EvaluationError.TypeError(value, cause, ctor.schema)
            )
        case EqualTo(left, right, tag) => for {
            leftValue  <- evaluate(left)
            rightValue <- evaluate(right)
          } yield tag.equal(leftValue, rightValue)
        case Math(operation, tag)      => operation match {
            case Math.Binary(left, right, operation) =>
              for {
                leftValue  <- evaluate(left)
                rightValue <- evaluate(right)
              } yield operation match {
                case Math.Binary.Add      => tag.add(leftValue, rightValue)
                case Math.Binary.Multiply => tag.multiply(leftValue, rightValue)
                case Math.Binary.Divide   => tag.divide(leftValue, rightValue)
                case Math.Binary.Modulo   => tag.modulo(leftValue, rightValue)
                case Math.Binary.GreaterThan =>
                  tag.greaterThan(leftValue, rightValue)
              }
            case Math.Unary(value, operation) => evaluate(value).map(evaluate =>
                operation match {
                  case Math.Unary.Negate => tag.negate(evaluate)
                }
              )
          }
        case Logical(operation)        => operation match {
            case Logical.Binary(left, right, operation) =>
              for {
                leftValue  <- evaluateAs[Boolean](left)
                rightValue <- evaluateAs[Boolean](right)
              } yield operation match {
                case Logical.Binary.And => leftValue && rightValue
                case Logical.Binary.Or  => leftValue || rightValue
              }
            case Logical.Unary(value, operation) => evaluateAs[Boolean](value)
                .flatMap { a =>
                  operation match {
                    case Logical.Unary.Not => ZIO.succeed(!a)
                    case Logical.Unary.Diverge(isTrue, isFalse) =>
                      if (a) evaluate(isTrue) else evaluate(isFalse)
                  }
                }
          }
      }
  }

  def live: ZLayer[EvaluationContext, Nothing, RemoteRuntime] =
    ZLayer.fromZIO(ZIO.service[EvaluationContext].map(ctx => new Default(ctx)))

  def evaluate[A](remote: Remote[A]) =
    ZIO.serviceWithZIO[RemoteRuntime](
      _.evaluateAs[A](remote.compile(CompilationContext.initial))
    )
}
