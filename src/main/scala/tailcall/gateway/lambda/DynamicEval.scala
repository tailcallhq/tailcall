package tailcall.gateway.lambda
import tailcall.gateway.ast.Endpoint
import zio.Chunk
import zio.schema.{DeriveSchema, DynamicValue, Schema}
sealed trait DynamicEval
object DynamicEval {
  // scalafmt: { maxColumn = 240 }
  final case class Literal(value: DynamicValue, ctor: Constructor[Any])                extends DynamicEval
  final case class Pipe(left: DynamicEval, right: DynamicEval)                         extends DynamicEval
  final case class FunctionDefinition(input: EvaluationContext.Key, body: DynamicEval) extends DynamicEval
  final case class Lookup(key: EvaluationContext.Key)                                  extends DynamicEval
  final case class Flatten(plan: DynamicEval)                                          extends DynamicEval
  final case class Debug(eval: DynamicEval, str: String)                               extends DynamicEval
  final case class Recurse(func: DynamicEval)                                          extends DynamicEval
  final case class EqualTo(left: DynamicEval, right: DynamicEval, tag: Equatable[Any]) extends DynamicEval
  final case class EndpointCall(endpoint: Endpoint, arg: DynamicEval)                  extends DynamicEval
  final case class Record(value: Chunk[(String, DynamicEval)])                         extends DynamicEval
  final case class Die(message: DynamicEval)                                           extends DynamicEval

  final case class Math(operation: Math.Operation, tag: Numeric[Any]) extends DynamicEval
  object Math {
    sealed trait Operation
    final case class Binary(left: DynamicEval, right: DynamicEval, operation: Binary.Operation) extends Operation
    object Binary {
      sealed trait Operation
      case object Add         extends Operation
      case object Multiply    extends Operation
      case object Divide      extends Operation
      case object Modulo      extends Operation
      case object GreaterThan extends Operation
    }
    final case class Unary(value: DynamicEval, operation: Unary.Operation) extends Operation
    object Unary  {
      sealed trait Operation
      case object Negate extends Operation
    }
  }

  final case class Logical(operation: Logical.Operation) extends DynamicEval
  object Logical {
    sealed trait Operation
    final case class Binary(left: DynamicEval, right: DynamicEval, operation: Binary.Operation) extends Operation
    object Binary {
      sealed trait Operation
      case object And extends Operation
      case object Or  extends Operation
    }
    final case class Unary(value: DynamicEval, operation: Unary.Operation) extends Operation
    object Unary  {
      sealed trait Operation
      case object Not                                                     extends Operation
      final case class Diverge(isTrue: DynamicEval, isFalse: DynamicEval) extends Operation
    }
  }

  final case class StringOperations(operation: StringOperations.Operation) extends DynamicEval
  object StringOperations {
    sealed trait Operation
    final case class Concat(left: DynamicEval, right: DynamicEval) extends Operation
  }

  final case class TupleOperations(operation: TupleOperations.Operation) extends DynamicEval
  object TupleOperations {
    sealed trait Operation
    final case class GetIndex(value: DynamicEval, index: Int) extends Operation
    final case class Cons(value: Chunk[DynamicEval])          extends Operation
  }

  final case class SeqOperations(operation: SeqOperations.Operation) extends DynamicEval
  // TODO: Support for other collections
  object SeqOperations {
    sealed trait Operation
    final case class Concat(left: DynamicEval, right: DynamicEval)               extends Operation
    final case class Reverse(seq: DynamicEval)                                   extends Operation
    final case class Filter(seq: DynamicEval, condition: DynamicEval)            extends Operation
    final case class FlatMap(seq: DynamicEval, operation: DynamicEval)           extends Operation
    final case class Length(seq: DynamicEval)                                    extends Operation
    final case class IndexOf(seq: DynamicEval, element: DynamicEval)             extends Operation
    final case class Slice(seq: DynamicEval, from: Int, to: Int)                 extends Operation
    final case class Head(seq: DynamicEval)                                      extends Operation
    final case class Sequence(value: Chunk[DynamicEval], ctor: Constructor[Any]) extends Operation
    final case class GroupBy(seq: DynamicEval, keyFunction: DynamicEval)         extends Operation
  }

  final case class MapOperations(operation: MapOperations.Operation) extends DynamicEval
  object MapOperations {
    sealed trait Operation
    final case class Get(map: DynamicEval, key: DynamicEval)        extends Operation
    final case class Cons(value: Chunk[(DynamicEval, DynamicEval)]) extends Operation
  }

  final case class EitherOperations(operation: EitherOperations.Operation) extends DynamicEval
  object EitherOperations {
    sealed trait Operation
    final case class Cons(value: Either[DynamicEval, DynamicEval])                   extends Operation
    final case class Fold(value: DynamicEval, left: DynamicEval, right: DynamicEval) extends Operation
  }

  final case class ContextOperations(context: DynamicEval, operation: ContextOperations.Operation) extends DynamicEval
  object ContextOperations {
    sealed trait Operation
    final case class GetArg(name: String) extends Operation
    case object GetValue                  extends Operation
    case object GetParent                 extends Operation
  }

  final case class OptionOperations(operation: OptionOperations.Operation) extends DynamicEval
  object OptionOperations {
    sealed trait Operation
    final case class Cons(option: Option[DynamicEval])                              extends Operation
    final case class Fold(value: DynamicEval, none: DynamicEval, some: DynamicEval) extends Operation
  }

  final case class DynamicValueOperations(value: DynamicEval, operation: DynamicValueOperations.Operation) extends DynamicEval
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

  implicit val schema: Schema[DynamicEval] = DeriveSchema.gen[DynamicEval]
}
