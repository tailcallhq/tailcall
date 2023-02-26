package tailcall.gateway.service

import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.lambda._
import zio._
import zio.schema.DynamicValue

trait EvaluationRuntime {
  final def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B] =
    evaluate(lambda.compile(CompilationContext.initial)).asInstanceOf[LExit[Any, Throwable, A, B]]

  def evaluate(dynamicEval: Expression[DynamicValue]): LExit[Any, Throwable, Any, Any]

  final def evaluateAs[A](eval: Expression[DynamicValue]): LExit[Any, Throwable, Any, A] =
    evaluate(eval).flatMap(a => LExit.attempt(a.asInstanceOf[A]))
}

object EvaluationRuntime {
  import Expression._

  def evaluate[A, B](ab: A ~> B): LExit[EvaluationRuntime, Throwable, A, B] =
    LExit.fromZIO(ZIO.service[EvaluationRuntime]).flatMap(_.evaluate(ab))

  def live: ZLayer[EvaluationContext, Nothing, EvaluationRuntime] =
    ZLayer.fromZIO(ZIO.service[EvaluationContext].map(new Live(_)))

  final class Live(ctx: EvaluationContext) extends EvaluationRuntime {

    override def evaluate(plan: Expression[DynamicValue]): LExit[Any, Throwable, Any, Any] = {
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
        case Lookup(binding)                   => LExit.fromZIO {
            for {
              ref <- ctx.get(binding)
              res <- ref match {
                case Some(value) => ZIO.succeed(value)
                case None        => ZIO.fail(EvaluationError.BindingNotFound(binding))
              }
            } yield res
          }

        case Immediate(eval0)   => for {
            eval1 <- evaluate(eval0)
            eval2 <- evaluate(eval1.asInstanceOf[Expression[DynamicValue]])
          } yield eval2
        case Defer(value)       => LExit.succeed(value)
        case Dynamic(operation) => for {
            input <- LExit.input[Any]
          } yield {
            val d = input.asInstanceOf[DynamicValue]
            operation match { case Dynamic.Typed(ctor) => DynamicValueUtil.as(d)(ctor.schema) }
          }
        case Dict(operation)    => operation match {
            case Dict.Get(key, map) => for {
                k <- evaluate(key)
                m <- evaluateAs[Map[Any, Any]](map)
              } yield m.get(k)
          }
        case Opt(operation)     => operation match {
            case Opt.IsSome                  => LExit.input.map(_.asInstanceOf[Option[_]].isDefined)
            case Opt.IsNone                  => LExit.input.map(_.asInstanceOf[Option[_]].isEmpty)
            case Opt.Fold(value, none, some) => for {
                opt <- evaluateAs[Option[_]](value)
                res <- opt match {
                  case Some(value) => evaluate(some).provideInput(value)
                  case None        => evaluate(none)
                }
              } yield res
          }
      }
    }
  }
}
