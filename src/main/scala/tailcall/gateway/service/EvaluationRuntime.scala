package tailcall.gateway.service

import tailcall.gateway.http.HttpClient
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.lambda._
import tailcall.gateway.remote.Remote
import zio._
import zio.schema.DynamicValue
import zio.schema.codec.JsonCodec

import java.nio.charset.StandardCharsets

trait EvaluationRuntime {
  final def evaluate[A](remote: Remote[A]): Task[A] = evaluate(remote.toLambda) {}

  final def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B] = evaluate(lambda, EvaluationContext.make)

  final def evaluate[A, B](lambda: A ~> B, ctx: EvaluationContext): LExit[Any, Throwable, A, B] =
    evaluate(lambda.compile(CompilationContext.initial), ctx).asInstanceOf[LExit[Any, Throwable, A, B]]

  def evaluate(dynamicEval: Expression, ctx: EvaluationContext): LExit[Any, Throwable, Any, Any]

  final def evaluateAs[A](eval: Expression, ctx: EvaluationContext): LExit[Any, Throwable, Any, A] =
    evaluate(eval, ctx).flatMap(a => LExit.attempt(a.asInstanceOf[A]))
}

object EvaluationRuntime {
  import Expression._

  def evaluate[A, B](ab: A ~> B): LExit[EvaluationRuntime, Throwable, A, B] =
    LExit.fromZIO(ZIO.service[EvaluationRuntime]).flatMap(_.evaluate(ab))

  def live: ZLayer[Any, Nothing, EvaluationRuntime] = ZLayer.succeed(new Live())

  final class Live() extends EvaluationRuntime {

    override def evaluate(plan: Expression, ctx: EvaluationContext): LExit[Any, Throwable, Any, Any] = {
      plan match {
        case Literal(value, schema)            => value.toTypedValue(schema) match {
            case Left(cause)  => LExit.fail(EvaluationError.TypeError(value, cause, schema))
            case Right(value) => LExit.succeed(value)
          }
        case EqualTo(left, right, tag)         => for {
            leftValue  <- evaluate(left, ctx)
            rightValue <- evaluate(right, ctx)
          } yield tag.equal(leftValue, rightValue)
        case Math(operation, tag)              => operation match {
            case Math.Binary(operation, left, right) =>
              for {
                leftValue  <- evaluate(left, ctx)
                rightValue <- evaluate(right, ctx)
              } yield operation match {
                case Math.Binary.Add              => tag.add(leftValue, rightValue)
                case Math.Binary.Multiply         => tag.multiply(leftValue, rightValue)
                case Math.Binary.Divide           => tag.divide(leftValue, rightValue)
                case Math.Binary.Modulo           => tag.modulo(leftValue, rightValue)
                case Math.Binary.GreaterThan      => tag.greaterThan(leftValue, rightValue)
                case Math.Binary.GreaterThanEqual => tag.greaterThanEqual(leftValue, rightValue)
              }
            case Math.Unary(operation, value)        =>
              for { value <- evaluate(value, ctx) } yield operation match { case Math.Unary.Negate => tag.negate(value) }
          }
        case Logical(operation)                => operation match {
            case Logical.Binary(operation, left, right) =>
              for {
                leftValue  <- evaluateAs[Boolean](left, ctx)
                rightValue <- evaluateAs[Boolean](right, ctx)
              } yield operation match {
                case Logical.Binary.And => leftValue && rightValue
                case Logical.Binary.Or  => leftValue || rightValue
              }
            case Logical.Unary(value, operation)        => evaluateAs[Boolean](value, ctx).flatMap { a =>
                operation match {
                  case Logical.Unary.Not                      => LExit.succeed(!a)
                  case Logical.Unary.Diverge(isTrue, isFalse) =>
                    if (a) evaluate(isTrue, ctx) else evaluate(isFalse, ctx)
                }
              }
          }
        case Identity                          => LExit.input
        case Pipe(left, right)                 => evaluate(left, ctx) >>> evaluate(right, ctx)
        case FunctionDef(binding, body, input) => for {
            i <- evaluate(input, ctx)
            r <- evaluate(body, ctx.set(binding, i))
          } yield r
        case Lookup(binding)                   => LExit.fromZIO {
            val ref = ctx.get(binding)
            for {
              res <- ref match {
                case Some(value) => ZIO.succeed(value)
                case None        => ZIO.fail(EvaluationError.BindingNotFound(binding))
              }
            } yield res
          }

        case Immediate(eval0)   => for {
            eval1 <- evaluateAs[Expression](eval0, ctx)
            eval2 <- evaluate(eval1, ctx)
          } yield eval2
        case Defer(value)       => LExit.succeed(value)
        case Dynamic(operation) => LExit.input[Any].map(input =>
            operation match {
              case Dynamic.Typed(schema)     => DynamicValueUtil.as(input.asInstanceOf[DynamicValue])(schema)
              case Dynamic.ToDynamic(schema) => schema.toDynamic(input)
              case Dynamic.Path(path)        => DynamicValueUtil.getPath(input.asInstanceOf[DynamicValue], path)
            }
          )
        case Dict(operation)    => operation match {
            case Dict.Get(key, map) => for {
                k <- evaluate(key, ctx)
                m <- evaluateAs[Map[Any, Any]](map, ctx)
              } yield m.get(k)
          }
        case Opt(operation)     => operation match {
            case Opt.IsSome                  => LExit.input.map(_.asInstanceOf[Option[_]].isDefined)
            case Opt.IsNone                  => LExit.input.map(_.asInstanceOf[Option[_]].isEmpty)
            case Opt.Fold(value, none, some) => for {
                opt <- evaluateAs[Option[_]](value, ctx)
                res <- opt match {
                  case Some(value) => evaluate(some, ctx).provideInput(value)
                  case None        => evaluate(none, ctx)
                }
              } yield res
            case Opt.Apply(value)            => value match {
                case None        => LExit.succeed(None)
                case Some(value) => for { any <- evaluate(value, ctx) } yield Option(any)
              }
          }
        case Unsafe(operation)  => operation match {
            case Unsafe.Die(message)           => LExit.fail(EvaluationError.Death(message))
            case Unsafe.Debug(prefix)          => for {
                input <- LExit.input[Any]
                _     <- LExit.fromZIO(Console.printLine(s"${prefix}: $input"))
              } yield input
            case Unsafe.EndpointCall(endpoint) => for {
                input <- LExit.input[Any]
                out   <- LExit.fromZIO {
                  for {
                    array <- ZIO.asyncInterrupt[Any, Throwable, Array[Byte]] { cb =>
                      val request = endpoint.evaluate(input.asInstanceOf[DynamicValue]).toHttpRequest
                      val close   = HttpClient.make.request(request)((status, _, body) =>
                        if (status >= 400) cb(ZIO.fail(new Throwable(s"HTTP Error: $status")))
                        else cb(ZIO.succeed(body))
                      )
                      Left(ZIO.succeed(close))
                    }
                    outputSchema = endpoint.outputSchema
                    any   <- ZIO.fromEither(
                      JsonCodec.jsonDecoder(outputSchema).decodeJson(new String(array, StandardCharsets.UTF_8))
                        .map(outputSchema.toDynamic)
                    ).mapError(EvaluationError.DecodingError)
                  } yield any
                }
              } yield out
          }
      }
    }
  }
}
