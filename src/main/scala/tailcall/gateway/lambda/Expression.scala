package tailcall.gateway.lambda

import tailcall.gateway.service.EvaluationContext.Binding
import zio.schema.{DeriveSchema, Schema}

// scalafmt: { maxColumn = 240 }
sealed trait Expression[+A]

object Expression {

  case object Identity                                                                         extends Expression[Nothing]
  final case class Defer[A](value: Expression[A])                                              extends Expression[A]
  final case class EqualTo[A](left: Expression[A], right: Expression[A], tag: Equatable[Any])  extends Expression[A]
  final case class FunctionDef[A](binding: Binding, body: Expression[A], input: Expression[A]) extends Expression[A]
  final case class Immediate[A](value: Expression[A])                                          extends Expression[A]
  final case class Literal[A](value: A, ctor: Constructor[Any])                                extends Expression[A]
  final case class Logical[A](operation: Logical.Operation[A])                                 extends Expression[A]
  final case class Lookup(binding: Binding)                                                    extends Expression[Nothing]
  final case class Math[A](operation: Math.Operation[A], tag: Numeric[Any])                    extends Expression[A]
  final case class Pipe[A](left: Expression[A], right: Expression[A])                          extends Expression[A]

  object Math {
    sealed trait Operation[+A]
    final case class Binary[A](operation: Binary.Operation, left: Expression[A], right: Expression[A]) extends Operation[A]
    final case class Unary[A](operation: Unary.Operation, value: Expression[A])                        extends Operation[A]

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
    sealed trait Operation[+A]
    final case class Binary[A](operation: Binary.Operation, left: Expression[A], right: Expression[A]) extends Operation[A]
    final case class Unary[A](value: Expression[A], operation: Unary.Operation[A])                     extends Operation[A]

    object Binary {
      sealed trait Operation
      case object And extends Operation
      case object Or  extends Operation
    }

    object Unary {
      sealed trait Operation[+A]
      final case class Diverge[A](isTrue: Expression[A], isFalse: Expression[A]) extends Operation[A]
      case object Not                                                            extends Operation[Nothing]
    }
  }

  final case class Dynamic(operation: Dynamic.Operation) extends Expression[Nothing]
  object Dynamic {
    sealed trait Operation
    case object AsSeq     extends Operation
    case object AsString  extends Operation
    case object AsInt     extends Operation
    case object AsBoolean extends Operation
    case object AsMap     extends Operation
  }

  implicit def schema[A](implicit a: Schema[A]): Schema[Expression[A]] = DeriveSchema.gen[Expression[A]]
}
