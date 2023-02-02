package tailcall.gateway.remote

import tailcall.gateway.remote.DynamicEval.Logical.{Binary, Unary}
import tailcall.gateway.remote.DynamicEval.StringOperations.Concat
import zio.schema.meta.MetaSchema
import zio.schema.{DeriveSchema, DynamicValue, Schema}

import java.util.concurrent.atomic.AtomicInteger

sealed trait DynamicEval

object DynamicEval {
  final case class Literal(value: DynamicValue, meta: MetaSchema) extends DynamicEval

  final case class EqualTo(left: DynamicEval, right: DynamicEval, tag: Equatable[Any])
      extends DynamicEval

  final case class Math(operation: Math.Operation, tag: Numeric[Any]) extends DynamicEval
  object Math {
    sealed trait Operation

    final case class Binary(left: DynamicEval, right: DynamicEval, operation: Binary.Operation)
        extends Operation
    object Binary {
      sealed trait Operation
      case object Add      extends Operation
      case object Multiply extends Operation
      case object Divide   extends Operation
      case object Modulo   extends Operation
    }

    case class Unary(value: DynamicEval, operation: Unary.Operation) extends Operation
    object Unary {
      sealed trait Operation
      case object Negate extends Operation
    }

    def apply(
      left: DynamicEval,
      right: DynamicEval,
      operation: Binary.Operation,
      tag: Numeric[Any]
    ): Math = Math(Binary(left, right, operation), tag)

    def apply(value: DynamicEval, operation: Unary.Operation, tag: Numeric[Any]): Math =
      Math(Unary(value, operation), tag)

  }

  final case class Logical(operation: Logical.Operation) extends DynamicEval
  object Logical {
    sealed trait Operation
    final case class Binary(left: DynamicEval, right: DynamicEval, operation: Binary.Operation)
        extends Operation

    object Binary {
      sealed trait Operation
      case object And extends Operation
      case object Or  extends Operation
    }

    final case class Unary(value: DynamicEval, operation: Unary.Operation) extends Operation
    object Unary {
      sealed trait Operation
      case object Not                                               extends Operation
      case class Diverge(isTrue: DynamicEval, isFalse: DynamicEval) extends Operation
    }

    def apply(left: DynamicEval, right: DynamicEval, operation: Binary.Operation): DynamicEval =
      Logical(Binary(left, right, operation))

    def apply(value: DynamicEval, operation: Unary.Operation): DynamicEval =
      Logical(Unary(value, operation))

  }

  final case class StringOperations(operation: StringOperations.Operation) extends DynamicEval
  object StringOperations {
    sealed trait Operation
    final case class Concat(left: DynamicEval, right: DynamicEval) extends Operation
  }

  final case class IndexSeqOperations(operation: IndexSeqOperations.Operation) extends DynamicEval

  object IndexSeqOperations {
    sealed trait Operation
    final case class Concat(left: DynamicEval, right: DynamicEval)     extends Operation
    final case class Reverse(seq: DynamicEval)                         extends Operation
    final case class Filter(seq: DynamicEval, condition: DynamicEval)  extends Operation
    final case class FlatMap(seq: DynamicEval, operation: DynamicEval) extends Operation
    final case class Map(seq: DynamicEval, operation: DynamicEval)     extends Operation
    final case class Length(seq: DynamicEval)                          extends Operation
    final case class IndexOf(seq: DynamicEval, element: DynamicEval)   extends Operation
  }

  final case class Apply(f: EvalFunction, arg: DynamicEval) extends DynamicEval
  final case class Binding private (id: Int)                extends DynamicEval
  object Binding {
    val counter       = new AtomicInteger(0)
    def make: Binding = new Binding(counter.incrementAndGet())
  }
  final case class EvalFunction(input: Binding, body: DynamicEval) extends DynamicEval

  def add(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Add, tag)

  def multiply(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Multiply, tag)

  def divide(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Divide, tag)

  def modulo(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Modulo, tag)

  def negate(value: DynamicEval, tag: Numeric[Any]): Math = Math(value, Math.Unary.Negate, tag)

  def and(left: DynamicEval, right: DynamicEval): DynamicEval =
    Logical(left, right, Logical.Binary.And)

  def or(left: DynamicEval, right: DynamicEval): DynamicEval =
    Logical(left, right, Logical.Binary.Or)

  def not(value: DynamicEval): DynamicEval = Logical(value, Logical.Unary.Not)

  def diverge(cond: DynamicEval, isTrue: DynamicEval, isFalse: DynamicEval): DynamicEval =
    Logical(Logical.Unary(cond, Logical.Unary.Diverge(isTrue, isFalse)))

  def equal(left: DynamicEval, right: DynamicEval, tag: Equatable[Any]): DynamicEval =
    EqualTo(left, right, tag.any)

  object Unsafe {
    def evaluateTyped[A](eval: DynamicEval): A = evaluate(eval).asInstanceOf[A]

    def evaluate(eval: DynamicEval): Any = eval match {
      case Literal(value, meta)          => value.toTypedValue(meta.toSchema) match {
          case Right(value) => value
          case Left(value)  => throw new RuntimeException("Could not translate literal: " + value)
        }
      case EqualTo(left, right, tag)     => tag.equal(evaluate(left), evaluate(right))
      case Math(operation, tag)          => operation match {
          case Math.Binary(left, right, operation) =>
            val leftValue  = evaluate(left)
            val rightValue = evaluate(right)
            operation match {
              case Math.Binary.Add      => tag.add(leftValue, rightValue)
              case Math.Binary.Multiply => tag.multiply(leftValue, rightValue)
              case Math.Binary.Divide   => tag.divide(leftValue, rightValue)
              case Math.Binary.Modulo   => tag.modulo(leftValue, rightValue)
            }
          case Math.Unary(value, operation)        =>
            val a = evaluate(value)
            operation match { case Math.Unary.Negate => tag.negate(a) }
        }
      case Logical(operation)            => operation match {
          case Binary(left, right, operation) =>
            val leftValue  = evaluateTyped[Boolean](left)
            val rightValue = evaluateTyped[Boolean](right)
            operation match {
              case Binary.And => leftValue && rightValue
              case Binary.Or  => leftValue || rightValue
            }
          case Unary(value, operation)        =>
            val a = evaluateTyped[Boolean](value)
            operation match {
              case Unary.Not                      => !a
              case Unary.Diverge(isTrue, isFalse) => if (a) evaluate(isTrue) else evaluate(isFalse)
            }
        }
      case StringOperations(operation)   => operation match {
          case Concat(left, right) => evaluateTyped[String](left) ++ evaluateTyped[String](right)
        }
      case IndexSeqOperations(operation) => ???
      case Apply(f, arg)                 => ???
      case Binding(id)                   => ???
      case EvalFunction(input, body)     => ???
    }
  }

  implicit val schema: Schema[DynamicEval] = DeriveSchema.gen[DynamicEval]
}
