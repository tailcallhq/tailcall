package tailcall.gateway.remote

import tailcall.gateway.ast.Endpoint
import zio.Chunk
import zio.schema.{DeriveSchema, DynamicValue, Schema}

import java.util.concurrent.atomic.AtomicInteger

sealed trait DynamicEval

object DynamicEval {
  final case class Literal(value: DynamicValue, ctor: Constructor[Any])
      extends DynamicEval

  final case class EqualTo(
    left: DynamicEval,
    right: DynamicEval,
    tag: Equatable[Any]
  ) extends DynamicEval

  final case class Math(operation: Math.Operation, tag: Numeric[Any])
      extends DynamicEval
  object Math {
    sealed trait Operation

    final case class Binary(
      left: DynamicEval,
      right: DynamicEval,
      operation: Binary.Operation
    ) extends Operation
    object Binary {
      sealed trait Operation
      case object Add         extends Operation
      case object Multiply    extends Operation
      case object Divide      extends Operation
      case object Modulo      extends Operation
      case object GreaterThan extends Operation
    }

    final case class Unary(value: DynamicEval, operation: Unary.Operation)
        extends Operation
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

    def apply(
      value: DynamicEval,
      operation: Unary.Operation,
      tag: Numeric[Any]
    ): Math = Math(Unary(value, operation), tag)

  }

  final case class Logical(operation: Logical.Operation) extends DynamicEval
  object Logical {
    sealed trait Operation
    final case class Binary(
      left: DynamicEval,
      right: DynamicEval,
      operation: Binary.Operation
    ) extends Operation

    object Binary {
      sealed trait Operation
      case object And extends Operation
      case object Or  extends Operation
    }

    final case class Unary(value: DynamicEval, operation: Unary.Operation)
        extends Operation
    object Unary {
      sealed trait Operation
      case object Not extends Operation
      final case class Diverge(isTrue: DynamicEval, isFalse: DynamicEval)
          extends Operation
    }

    def apply(
      left: DynamicEval,
      right: DynamicEval,
      operation: Binary.Operation
    ): DynamicEval = Logical(Binary(left, right, operation))

    def apply(value: DynamicEval, operation: Unary.Operation): DynamicEval =
      Logical(Unary(value, operation))

  }

  final case class StringOperations(operation: StringOperations.Operation)
      extends DynamicEval
  object StringOperations {
    sealed trait Operation
    final case class Concat(left: DynamicEval, right: DynamicEval)
        extends Operation
  }
  final case class TupleOperations(operation: TupleOperations.Operation)
      extends DynamicEval
  object TupleOperations  {
    sealed trait Operation
    final case class GetIndex(value: DynamicEval, index: Int) extends Operation
    final case class Cons(value: Chunk[DynamicEval])          extends Operation
  }

  final case class SeqOperations(operation: SeqOperations.Operation)
      extends DynamicEval

  // TODO: rename to SeqOperations
  // TODO: Support for other collections
  object SeqOperations {
    sealed trait Operation
    final case class Concat(left: DynamicEval, right: DynamicEval)
        extends Operation
    final case class Reverse(seq: DynamicEval)           extends Operation
    final case class Filter(seq: DynamicEval, condition: EvalFunction)
        extends Operation
    final case class FlatMap(seq: DynamicEval, operation: EvalFunction)
        extends Operation
    final case class Length(seq: DynamicEval)            extends Operation
    final case class IndexOf(seq: DynamicEval, element: DynamicEval)
        extends Operation
    final case class Slice(seq: DynamicEval, from: Int, to: Int)
        extends Operation
    final case class Head(seq: DynamicEval)              extends Operation
    final case class Sequence(value: Chunk[DynamicEval]) extends Operation
    final case class GroupBy(seq: DynamicEval, keyFunction: EvalFunction)
        extends Operation
  }

  final case class MapOperations(operation: MapOperations.Operation)
      extends DynamicEval
  object MapOperations {
    sealed trait Operation
    final case class Get(map: DynamicEval, key: DynamicEval) extends Operation
    final case class Cons(value: Chunk[(DynamicEval, DynamicEval)])
        extends Operation
  }

  final case class FunctionCall(f: EvalFunction, arg: DynamicEval)
      extends DynamicEval
  final case class Binding(id: Int) extends DynamicEval
  object Binding {
    private val counter = new AtomicInteger(0)
    def make: Binding   = new Binding(counter.incrementAndGet())
  }

  final case class EitherOperations(operation: EitherOperations.Operation)
      extends DynamicEval
  object EitherOperations  {
    sealed trait Operation
    final case class Cons(value: Either[DynamicEval, DynamicEval])
        extends Operation
    final case class Fold(
      value: DynamicEval,
      left: EvalFunction,
      right: EvalFunction
    ) extends Operation
  }

  final case class ContextOperations(
    context: DynamicEval,
    operation: ContextOperations.Operation
  ) extends DynamicEval
  object ContextOperations {
    sealed trait Operation
    final case class GetArg(name: String) extends Operation
    case object GetValue                  extends Operation
    case object GetParent                 extends Operation
  }

  final case class OptionOperations(operation: OptionOperations.Operation)
      extends DynamicEval

  object OptionOperations {
    sealed trait Operation
    final case class Cons(option: Option[DynamicEval]) extends Operation
    final case class Fold(
      value: DynamicEval,
      none: DynamicEval,
      some: EvalFunction
    ) extends Operation
  }

  final case class EvalFunction(input: Binding, body: DynamicEval)
      extends DynamicEval

  final case class EndpointCall(endpoint: Endpoint, arg: DynamicEval)
      extends DynamicEval

  final case class Record(value: Chunk[(String, DynamicEval)])
      extends DynamicEval

  final case class Die(message: DynamicEval) extends DynamicEval

  final case class DynamicValueOperations(
    value: DynamicEval,
    operation: DynamicValueOperations.Operation
  ) extends DynamicEval

  object DynamicValueOperations {
    sealed trait Operation
    final case class Path(path: Chunk[String]) extends Operation
    case object AsString                       extends Operation
    case object AsBoolean                      extends Operation
    case object AsInt                          extends Operation
    case object AsLong                         extends Operation
    case object AsDouble                       extends Operation
    case object AsFloat                        extends Operation
    case object AsBigDecimal                   extends Operation
    case object AsList                         extends Operation
    case object AsMap                          extends Operation
  }

  final case class Debug(eval: DynamicEval, str: String) extends DynamicEval

  final case class Recurse(func: EvalFunction) extends DynamicEval

  final case class Flatten(eval: DynamicEval) extends DynamicEval

  def add(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Add, tag)

  def multiply(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Multiply, tag)

  def divide(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Divide, tag)

  def modulo(left: DynamicEval, right: DynamicEval, tag: Numeric[Any]): Math =
    Math(left, right, Math.Binary.Modulo, tag)

  def greaterThan(
    left: DynamicEval,
    right: DynamicEval,
    tag: Numeric[Any]
  ): Math = Math(left, right, Math.Binary.GreaterThan, tag)

  def negate(value: DynamicEval, tag: Numeric[Any]): Math =
    Math(value, Math.Unary.Negate, tag)

  def and(left: DynamicEval, right: DynamicEval): DynamicEval =
    Logical(left, right, Logical.Binary.And)

  def or(left: DynamicEval, right: DynamicEval): DynamicEval =
    Logical(left, right, Logical.Binary.Or)

  def not(value: DynamicEval): DynamicEval = Logical(value, Logical.Unary.Not)

  def diverge(
    cond: DynamicEval,
    isTrue: DynamicEval,
    isFalse: DynamicEval
  ): DynamicEval =
    Logical(Logical.Unary(cond, Logical.Unary.Diverge(isTrue, isFalse)))

  def equal(
    left: DynamicEval,
    right: DynamicEval,
    tag: Equatable[Any]
  ): DynamicEval = EqualTo(left, right, tag.any)

  def binding: Binding = Binding.make

  def call(f: EvalFunction, arg: DynamicEval): DynamicEval =
    FunctionCall(f, arg)

  def filter(seq: DynamicEval, condition: EvalFunction): DynamicEval =
    SeqOperations(SeqOperations.Filter(seq, condition))

  def flatMap(seq: DynamicEval, operation: EvalFunction): DynamicEval =
    SeqOperations(SeqOperations.FlatMap(seq, operation))

  def concat(left: DynamicEval, right: DynamicEval): DynamicEval =
    SeqOperations(SeqOperations.Concat(left, right))

  def mapGet(map: DynamicEval, key: DynamicEval): DynamicEval =
    MapOperations(MapOperations.Get(map, key))

  def reverse(seq: DynamicEval): DynamicEval =
    SeqOperations(SeqOperations.Reverse(seq))

  def length(seq: DynamicEval): DynamicEval =
    SeqOperations(SeqOperations.Length(seq))

  def find(seq: DynamicEval, condition: EvalFunction): DynamicEval = ???

  def indexOf(seq: DynamicEval, element: DynamicEval): DynamicEval =
    SeqOperations(SeqOperations.IndexOf(seq, element))

  def take(seq: DynamicEval, n: Int): DynamicEval = slice(seq, 0, n)

  def slice(seq: DynamicEval, from: Int, to: Int): DynamicEval =
    SeqOperations(SeqOperations.Slice(seq, from, to))

  def head(seq: DynamicEval): DynamicEval =
    SeqOperations(SeqOperations.Head(seq))

  def groupBy(seq: DynamicEval, keyFunction: EvalFunction): DynamicEval =
    SeqOperations(SeqOperations.GroupBy(seq, keyFunction))

  def foldEither(
    value: DynamicEval,
    left: EvalFunction,
    right: EvalFunction
  ): DynamicEval = EitherOperations(EitherOperations.Fold(value, left, right))

  def concatStrings(left: DynamicEval, right: DynamicEval): DynamicEval =
    StringOperations(StringOperations.Concat(left, right))

  def seq(a: Seq[DynamicEval]): DynamicEval =
    SeqOperations(SeqOperations.Sequence(Chunk.fromIterable(a)))

  def map(a: Map[DynamicEval, DynamicEval]): DynamicEval =
    MapOperations(MapOperations.Cons(Chunk.fromIterable(a)))

  def either(a: Either[DynamicEval, DynamicEval]): DynamicEval =
    EitherOperations(EitherOperations.Cons(a))

  def bind(input: Binding, body: DynamicEval): DynamicEval =
    EvalFunction(input, body)

  def contextValue(context: DynamicEval): DynamicEval =
    ContextOperations(context, ContextOperations.GetValue)

  def contextArgs(context: DynamicEval, name: String): DynamicEval =
    ContextOperations(context, ContextOperations.GetArg(name))

  def contextParent(context: DynamicEval): DynamicEval =
    ContextOperations(context, ContextOperations.GetParent)

  def foldOption(
    value: DynamicEval,
    none: DynamicEval,
    some: EvalFunction
  ): DynamicEval = OptionOperations(OptionOperations.Fold(value, none, some))

  def option(value: Option[DynamicEval]): DynamicEval =
    OptionOperations(OptionOperations.Cons(value))

  def none: DynamicEval = OptionOperations(OptionOperations.Cons(None))

  def endpoint(endpoint: Endpoint, input: DynamicEval): DynamicEval =
    EndpointCall(endpoint, input)

  def record(fields: Seq[(String, DynamicEval)]): DynamicEval =
    Record(Chunk.fromIterable(fields))

  def die(message: DynamicEval): DynamicEval = Die(message)

  def dynamicValuePath(value: DynamicEval, path: Chunk[String]): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.Path(path))

  def dynamicValueAsString(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsString)

  def dynamicValueAsBoolean(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsBoolean)

  def dynamicValueAsInt(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsInt)

  def dynamicValueAsLong(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsLong)

  def dynamicValueAsDouble(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsDouble)

  def dynamicValueAsFloat(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsFloat)

  def dynamicValueAsList(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsList)

  def dynamicValueAsMap(value: DynamicEval): DynamicEval =
    DynamicValueOperations(value, DynamicValueOperations.AsMap)

  def debug(compile: DynamicEval, message: String): DynamicEval =
    Debug(compile, message)

  def tuple(value: Chunk[DynamicEval]): DynamicEval =
    DynamicEval.TupleOperations(TupleOperations.Cons(value))

  def tupleIndex(seq: DynamicEval, index: Int): DynamicEval =
    TupleOperations(TupleOperations.GetIndex(seq, index))

  def cons[A](value: DynamicValue, ctor: Constructor[A]): DynamicEval =
    Literal(value, ctor.asInstanceOf[Constructor[Any]])

  def recurse(func: EvalFunction): DynamicEval = Recurse(func)

  def flatten(r: DynamicEval): DynamicEval = Flatten(r)

  implicit val schema: Schema[DynamicEval] = DeriveSchema.gen[DynamicEval]
}
