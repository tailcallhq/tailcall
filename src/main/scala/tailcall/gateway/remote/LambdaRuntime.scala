package tailcall.gateway.remote

import tailcall.gateway.ast.Context
import tailcall.gateway.http.HttpClient
import tailcall.gateway.internal.ChunkUtil
import zio._
import zio.schema.codec.JsonCodec
import zio.schema.{DynamicValue, Schema, TypeId}

import java.nio.charset.StandardCharsets
import scala.collection.immutable.ListMap

trait LambdaRuntime {
  def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B]
}

object LambdaRuntime {
  import DynamicEval._

  final class Live(ctx: EvaluationContext) extends LambdaRuntime {
    def evaluate[A, B](lambda: A ~> B): LExit[Any, Throwable, A, B] =
      evaluate(lambda.compile(CompilationContext.initial))
        .asInstanceOf[LExit[Any, Throwable, A, B]]

    def evaluateAs[A](eval: DynamicEval): LExit[Any, Throwable, Any, A] =
      evaluate(eval).flatMap(a => LExit.attempt(a.asInstanceOf[A]))

    def evaluate(plan: DynamicEval): LExit[Any, Throwable, Any, Any] = {
      plan match {
        case FunctionOperations(operation) => operation match {
            case FunctionOperations.Literal(value, ctor) =>
              ctor.schema.fromDynamic(value) match {
                case Left(cause)  => LExit
                    .fail(EvaluationError.TypeError(value, cause, ctor.schema))
                case Right(value) => LExit.succeed(value)
              }

            case FunctionOperations.Pipe(left, right) =>
              evaluate(left) >>> evaluate(right)

            case FunctionOperations.Lookup(key) => LExit.fromZIO(
                ctx.get(key).mapError(_ => EvaluationError.BindingNotFound(key))
              )

            case FunctionOperations.FunctionDefinition(key, body) => for {
                any <- LExit.input[Any]
                _   <- LExit.fromZIO(ctx.set(key, any))
                res <- evaluate(body)
                _   <- LExit.fromZIO(ctx.drop(key))
              } yield res

            case FunctionOperations.Flatten(eval) => for {
                inner <- evaluate(eval)
                outer <- evaluate(
                  inner
                    .asInstanceOf[Lambda[_, _]]
                    .compile(CompilationContext.initial)
                )
              } yield outer
          }
        case Literal(value, ctor)          => value
            .toTypedValue(ctor.schema)
            .fold(
              cause =>
                LExit
                  .fail(EvaluationError.TypeError(value, cause, ctor.schema)),
              LExit.succeed
            )

        case EqualTo(left, right, tag)   => for {
            leftValue  <- evaluate(left)
            rightValue <- evaluate(right)
          } yield tag.equal(leftValue, rightValue)
        case Math(operation, tag)        => operation match {
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
        case Logical(operation)          => operation match {
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
                    case Logical.Unary.Not => LExit.succeed(!a)
                    case Logical.Unary.Diverge(isTrue, isFalse) =>
                      if (a) evaluate(isTrue) else evaluate(isFalse)
                  }
                }
          }
        case StringOperations(operation) => operation match {
            case StringOperations.Concat(left, right) => for {
                leftValue  <- evaluateAs[String](left)
                rightValue <- evaluateAs[String](right)
              } yield leftValue ++ rightValue
          }
        case MapOperations(operation)    => operation match {
            case MapOperations.Get(map, key) => for {
                map <- evaluateAs[Map[Any, Any]](map)
                key <- evaluateAs[Any](key)
              } yield map.get(key)
            case MapOperations.Cons(values)  =>
              val result = LExit.foreach(values) { case (key, value) =>
                evaluate(value).map(key -> _)
              }
              result.map(_.toMap)
          }
        case SeqOperations(operation)    => operation match {
            case SeqOperations.Concat(left, right)    => for {
                leftValue  <- evaluateAs[Seq[_]](left)
                rightValue <- evaluateAs[Seq[_]](right)
              } yield leftValue ++ rightValue
            case SeqOperations.IndexOf(seq, element)  => for {
                seq <- evaluateAs[Seq[_]](seq)
                e   <- evaluate(element)
              } yield seq.indexOf(e)
            case SeqOperations.Reverse(seq)           =>
              evaluateAs[Seq[_]](seq).map(_.reverse)
            case SeqOperations.Filter(seq, condition) => for {
                seq    <- evaluateAs[Seq[_]](seq)
                result <-
                  LExit.filter(seq)(any => call[Boolean](condition, any))
              } yield result

            case SeqOperations.FlatMap(seq, operation)   => for {
                seq    <- evaluateAs[Seq[Any]](seq)
                result <- ZIO.foreach(seq)(any => call[Seq[_]](operation, any))
              } yield result.flatten
            case SeqOperations.Length(seq)               =>
              evaluateAs[Seq[_]](seq).map(_.length)
            case SeqOperations.Slice(seq, from, to)      => for {
                seq    <- evaluateAs[Seq[_]](seq)
                result <- LExit.succeed(seq.slice(from, to))
              } yield result
            case SeqOperations.Head(seq)                 =>
              evaluateAs[Seq[_]](seq).map(_.headOption)
            case SeqOperations.Sequence(value, _)        =>
              LExit.foreach(value)(evaluate)
            case SeqOperations.GroupBy(seq, keyFunction) => for {
                seq <- evaluateAs[Seq[Any]](seq)
                map <- LExit.foreach(seq)(any =>
                  call[Any](keyFunction, any).map(_ -> any)
                )
              } yield map.groupBy(_._1).map { case (k, v) => k -> v.map(_._2) }
          }
        case EitherOperations(operation) => operation match {
            case EitherOperations.Cons(value)              => value match {
                case Left(value)  => evaluate(value).map(Left(_))
                case Right(value) => evaluate(value).map(Right(_))
              }
            case EitherOperations.Fold(value, left, right) => for {
                either <- evaluateAs[Either[_, _]](value)
                result <- either match {
                  case Left(value)  => call[Any](left, value)
                  case Right(value) => call[Any](right, value)
                }
              } yield result
          }
        case OptionOperations(operation) => operation match {
            case OptionOperations.Cons(option)            => option match {
                case Some(value) => evaluate(value).map(Some(_))
                case None        => LExit.none
              }
            case OptionOperations.Fold(value, none, some) => for {
                option <- evaluateAs[Option[_]](value)
                result <- option match {
                  case Some(value) => call(some, value)
                  case None        => evaluate(none)
                }
              } yield result
          }

        case Die(message)   => evaluateAs[String](message)
            .flatMap(message => LExit.fail(EvaluationError.Death(message)))
        case Record(fields) => for {
            f <- LExit.foreach(fields)(field =>
              evaluateAs[DynamicValue](field._2).map(field._1 -> _)
            )
          } yield DynamicValue.Record(TypeId.Structural, ListMap.from(f))

        case TupleOperations(operations) => operations match {
            case TupleOperations.Cons(values)       => for {
                any <- LExit.foreach(values)(evaluate)
                tup <- ChunkUtil.toTuple(any) match {
                  case null    =>
                    LExit.fail(EvaluationError.InvalidTupleSize(any.length))
                  case product => LExit.succeed(product)
                }
              } yield tup
            case TupleOperations.GetIndex(value, i) => for {
                f <- evaluateAs[Product](value)
              } yield f.productIterator.toSeq(i)
          }

        case ContextOperations(self, operation) => evaluateAs[Context](self)
            .map { ctx =>
              operation match {
                case ContextOperations.GetArg(name) => ctx.args.get(name)
                case ContextOperations.GetValue     => ctx.value
                case ContextOperations.GetParent    => ctx.parent
              }
            }

        case EndpointCall(endpoint, arg) => for {
            input <- evaluateAs[DynamicValue](arg)
            req = endpoint.evaluate(input).toHttpRequest
            array <- LExit.fromZIO(ZIO.async[Any, Nothing, Array[Byte]](cb =>
              HttpClient
                .make
                .request(req)((_, _, body) => cb(ZIO.succeed(body)))
            ))
            outputSchema = endpoint.outputSchema.asInstanceOf[Schema[Any]]
            any <- LExit
              .fromEither(
                JsonCodec
                  .jsonDecoder(outputSchema)
                  .decodeJson(new String(array, StandardCharsets.UTF_8))
                  .map(outputSchema.toDynamic)
              )
              .mapError(EvaluationError.DecodingError)
          } yield any

        case _: DynamicValueOperations => ???

        case Debug(self, prefix) => evaluate(self).debug(prefix)

        case Recurse(_) => ???

        case Flatten(eval) => evaluateAs[Remote[_]](eval).flatMap(evaluate(_))
      }
    }
  }

  def live: ZLayer[EvaluationContext, Nothing, LambdaRuntime] =
    ZLayer.fromZIO(ZIO.service[EvaluationContext].map(new Live(_)))

  def evaluate[A, B](ab: A ~> B): LExit[LambdaRuntime, Throwable, A, B] =
    LExit.fromZIO(ZIO.service[LambdaRuntime]).flatMap(_.evaluate(ab))
}
