package tailcall.runtime.lambda

import tailcall.runtime.JsonT
import tailcall.runtime.model.Endpoint
import tailcall.runtime.service.EvaluationContext.Binding
import zio.json.JsonCodec
import zio.schema.meta.MetaSchema
import zio.schema.{DeriveSchema, DynamicValue, Schema}
import zio.{UIO, ZIO}

sealed trait Expression {
  self =>
  def collect[A](pf: PartialFunction[Expression, A]): List[A] = Expression.collect(self, pf.lift)
}

object Expression {

  case object Identity                                                                extends Expression
  final case class Defer(value: Expression)                                           extends Expression
  final case class EqualTo(left: Expression, right: Expression, tag: Equatable.Tag)   extends Expression
  final case class FunctionDef(binding: Binding, body: Expression, input: Expression) extends Expression
  final case class Immediate(value: Expression)                                       extends Expression
  final case class Literal(value: DynamicValue, schema: MetaSchema)                   extends Expression
  final case class Logical(operation: Logical.Operation)                              extends Expression
  final case class Lookup(binding: Binding)                                           extends Expression
  final case class Math(operation: Math.Operation, tag: Numeric.Tag)                  extends Expression
  final case class Pipe(left: Expression, right: Expression)                          extends Expression

  // TODO: Lambda should not have any unsafe operations
  final case class Unsafe(operation: Unsafe.Operation) extends Expression
  object Unsafe {
    sealed trait Operation
    final case class Debug(prefix: String)                      extends Operation
    final case class EndpointCall(endpoint: Endpoint)           extends Operation
    final case class Tap(self: Expression, f: Any => UIO[Unit]) extends Operation
    object Tap {
      implicit val schema: Schema[Tap] = Schema[Unit].transform[Tap](_ => Tap(Identity, _ => ZIO.unit), _ => ())
    }
    final case class BatchEndpointCall(endpoint: Endpoint, join: Expression) extends Operation
  }

  object Math {
    sealed trait Operation
    final case class Binary(operation: Binary.Operation, left: Expression, right: Expression) extends Operation
    final case class Unary(operation: Unary.Operation, value: Expression)                     extends Operation

    object Binary {
      sealed trait Operation
      case object Add              extends Operation
      case object Multiply         extends Operation
      case object Divide           extends Operation
      case object Modulo           extends Operation
      case object GreaterThan      extends Operation
      case object GreaterThanEqual extends Operation
    }

    object Unary {
      sealed trait Operation
      case object Negate extends Operation
    }
  }

  object Logical {
    sealed trait Operation
    final case class Binary(operation: Binary.Operation, left: Expression, right: Expression) extends Operation
    final case class Unary(value: Expression, operation: Unary.Operation)                     extends Operation

    object Binary {
      sealed trait Operation
      case object And extends Operation
      case object Or  extends Operation
    }

    object Unary {
      sealed trait Operation
      final case class Diverge(isTrue: Expression, isFalse: Expression) extends Operation
      case object Not                                                   extends Operation
    }
  }

  final case class Dynamic(operation: Dynamic.Operation) extends Expression
  object Dynamic {
    sealed trait Operation
    final case class Typed(ctor: MetaSchema)                    extends Operation
    final case class Path(name: List[String], nestSeq: Boolean) extends Operation
    final case class ToDynamic(ctor: MetaSchema)                extends Operation
    final case class JsonTransform(jsonT: JsonT)                extends Operation
  }

  final case class Dict(operation: Dict.Operation) extends Expression
  object Dict {
    sealed trait Operation
    final case class Get(key: Expression, map: Expression)                    extends Operation
    final case class Put(key: Expression, value: Expression, map: Expression) extends Operation
    case object ToPair                                                        extends Operation
  }

  final case class Opt(operation: Opt.Operation) extends Expression
  object Opt {
    sealed trait Operation
    case object IsSome                                                           extends Operation
    case object IsNone                                                           extends Operation
    final case class Fold(value: Expression, none: Expression, some: Expression) extends Operation
    final case class Apply(value: Option[Expression])                            extends Operation
    final case class ToSeq(value: Expression)                                    extends Operation
  }

  final case class Sequence(value: Expression, operation: Sequence.Operation) extends Expression
  object Sequence {
    sealed trait Operation
    final case object MakeString            extends Operation
    final case class Map(f: Expression)     extends Operation
    final case class FlatMap(f: Expression) extends Operation
  }

  final case class Str(self: Expression, operation: Str.Operation) extends Expression
  object Str {
    sealed trait Operation
    final case class Concat(other: Expression) extends Operation
  }

  implicit val schema: Schema[Expression]       = DeriveSchema.gen[Expression]
  implicit val jsonCodec: JsonCodec[Expression] = zio.schema.codec.JsonCodec.jsonCodec(schema)

  def collect[A](expr: Expression, f: Expression => Option[A]): List[A] = {
    expr match {
      case Expression.Identity                    => f(expr).toList
      case Expression.Defer(value)                => collect(value, f)
      case Expression.EqualTo(left, right, _)     => collect(left, f) ++ collect(right, f)
      case Expression.FunctionDef(_, body, input) => collect(body, f) ++ collect(input, f)
      case Expression.Immediate(value)            => collect(value, f)
      case Expression.Literal(_, _)               => f(expr).toList
      case Expression.Logical(operation)          => operation match {
          case Logical.Binary(_, left, right)  => collect(left, f) ++ collect(right, f)
          case Logical.Unary(value, operation) => operation match {
              case Logical.Unary.Diverge(isTrue, isFalse) =>
                collect(value, f) ++ collect(isTrue, f) ++ collect(isFalse, f)
              case Logical.Unary.Not                      => collect(value, f)
            }
        }
      case Expression.Lookup(_)                   => f(expr).toList
      case Expression.Math(operation, _)          => operation match {
          case Math.Binary(_, left, right) => collect(left, f) ++ collect(right, f)
          case Math.Unary(_, value)        => collect(value, f)
        }
      case Expression.Pipe(left, right)           => collect(left, f) ++ collect(right, f)
      case Expression.Unsafe(operation)           => operation match {
          case Unsafe.Debug(_)                   => f(expr).toList
          case Unsafe.EndpointCall(_)            => f(expr).toList
          case Unsafe.BatchEndpointCall(_, join) => f(expr).toList ++ collect(join, f)
          case Unsafe.Tap(self, _)               => collect(self, f)
        }
      case Expression.Dynamic(_)                  => f(expr).toList
      case Expression.Dict(operation)             => operation match {
          case Dict.Get(key, map)        => collect(key, f) ++ collect(map, f)
          case Dict.Put(key, value, map) => collect(key, f) ++ collect(value, f) ++ collect(map, f)
          case Dict.ToPair               => f(expr).toList
        }
      case Expression.Opt(operation)              => operation match {
          case Opt.IsSome                  => f(expr).toList
          case Opt.IsNone                  => f(expr).toList
          case Opt.Fold(value, none, some) => collect(value, f) ++ collect(none, f) ++ collect(some, f)
          case Opt.Apply(value)            => value match {
              case Some(value) => collect(value, f)
              case None        => f(expr).toList
            }
          case Opt.ToSeq(value)            => collect(value, f)
        }
      case Expression.Sequence(value, operation)  => operation match {
          case Sequence.MakeString    => collect(value, f)
          case Sequence.Map(func)     => collect(value, f) ++ collect(func, f)
          case Sequence.FlatMap(func) => collect(value, f) ++ collect(func, f)
        }
      case Expression.Str(self, operation)        =>
        operation match { case Str.Concat(other) => collect(self, f) ++ collect(other, f) }
    }
  }
}
