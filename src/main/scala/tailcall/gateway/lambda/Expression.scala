package tailcall.gateway.lambda

import tailcall.gateway.service.EvaluationContext.Binding
import zio.schema._
import zio.schema.meta.MetaSchema

// scalafmt: { maxColumn = 240 }
sealed trait Expression

object Expression:
  // COMMENTED MARKERS ARE FOR FUTURE USE
  given schema: Schema[Expression] = DeriveSchema.gen[Expression]
  case object Identity                                                                extends Expression
  final case class Defer(value: Expression)                                           extends Expression
  final case class EqualTo(left: Expression, right: Expression, tag: Equatable.Tag)   extends Expression
  final case class FunctionDef(binding: Binding, body: Expression, input: Expression) extends Expression
  final case class Immediate(value: Expression)                                       extends Expression
  final case class Literal(value: DynamicValue, schema: MetaSchema)                   extends Expression

  final case class Lookup(binding: Binding) extends Expression

  final case class Pipe(left: Expression, right: Expression) extends Expression
  final case class Die(reason: String)                       extends Expression
  final case class Debug(prefix: String)                     extends Expression

  final case class Math(operation: Math.Operation, tag: Numeric) extends Expression
  object Math:
    sealed trait Operation
    final case class Binary(operation: Binary.Operation, left: Expression, right: Expression) extends Operation
    final case class Unary(operation: Unary.Operation, value: Expression)                     extends Operation

    object Binary:
      sealed trait Operation
      case object Add              extends Operation
      case object Multiply         extends Operation
      case object Divide           extends Operation
      case object Modulo           extends Operation
      case object GreaterThan      extends Operation
      case object GreaterThanEqual extends Operation

    object Unary:
      sealed trait Operation
      case object Negate extends Operation

  final case class Logical(operation: Logical.Operation) extends Expression
  object Logical:
    sealed trait Operation
    final case class Binary(operation: Binary.Operation, left: Expression, right: Expression) extends Operation
    final case class Unary(value: Expression, operation: Unary.Operation)                     extends Operation

    object Binary:
      sealed trait Operation
      case object And extends Operation
      case object Or  extends Operation

    object Unary:
      sealed trait Operation
      final case class Diverge(isTrue: Expression, isFalse: Expression) extends Operation
      case object Not                                                   extends Operation

  final case class Dynamic(operation: Dynamic.Operation) extends Expression
  object Dynamic:
    sealed trait Operation
    final case class Typed(ctor: MetaSchema)     extends Operation
    final case class Path(name: List[String])    extends Operation
    final case class ToDynamic(ctor: MetaSchema) extends Operation

  final case class Dict(operation: Dict.Operation) extends Expression
  object Dict:
    sealed trait Operation
    final case class Get(key: Expression, map: Expression) extends Operation

  final case class Opt(operation: Opt.Operation) extends Expression
  object Opt:
    sealed trait Operation
    case object IsSome                                                           extends Operation
    case object IsNone                                                           extends Operation
    final case class Fold(value: Expression, none: Expression, some: Expression) extends Operation
    final case class Apply(value: Option[Expression])                            extends Operation
