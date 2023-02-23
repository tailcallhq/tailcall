package tailcall.gateway.lambda

import tailcall.gateway.service.EvaluationContext.Binding
import zio.schema.{DeriveSchema, Schema}

sealed trait DynamicEval[+A]

object DynamicEval {
  // scalafmt: { maxColumn = 240 }
  case object Identity                                                                           extends DynamicEval[Nothing]
  final case class Lookup(binding: Binding)                                                      extends DynamicEval[Nothing]
  final case class Immediate[A](value: DynamicEval[A])                                           extends DynamicEval[A]
  final case class Defer[A](value: DynamicEval[A])                                               extends DynamicEval[A]
  final case class FunctionDef[A](binding: Binding, body: DynamicEval[A], input: DynamicEval[A]) extends DynamicEval[A]
  final case class Literal[A](value: A, ctor: Constructor[Any])                                  extends DynamicEval[A]
  final case class Pipe[A](left: DynamicEval[A], right: DynamicEval[A])                          extends DynamicEval[A]
  final case class EqualTo[A](left: DynamicEval[A], right: DynamicEval[A], tag: Equatable[Any])  extends DynamicEval[A]
  final case class Math[A](operation: Math.Operation[A], tag: Numeric[Any])                      extends DynamicEval[A]
  object Math {
    sealed trait Operation[+A]
    final case class Binary[A](operation: Binary.Operation, left: DynamicEval[A], right: DynamicEval[A]) extends Operation[A]
    object Binary {
      sealed trait Operation
      case object Add              extends Operation
      case object Multiply         extends Operation
      case object Divide           extends Operation
      case object Modulo           extends Operation
      case object GreaterThan      extends Operation
      case object GreaterThanEqual extends Operation
    }

    final case class Unary[A](operation: Unary.Operation, value: DynamicEval[A]) extends Operation[A]
    object Unary {
      sealed trait Operation
      case object Negate extends Operation
    }
  }

  final case class Logical[A](operation: Logical.Operation[A]) extends DynamicEval[A]
  object Logical {
    sealed trait Operation[+A]
    final case class Binary[A](operation: Binary.Operation, left: DynamicEval[A], right: DynamicEval[A]) extends Operation[A]
    object Binary {
      sealed trait Operation
      case object And extends Operation
      case object Or  extends Operation
    }

    final case class Unary[A](value: DynamicEval[A], operation: Unary.Operation[A]) extends Operation[A]
    object Unary {
      sealed trait Operation[+A]
      case object Not                                                              extends Operation[Nothing]
      final case class Diverge[A](isTrue: DynamicEval[A], isFalse: DynamicEval[A]) extends Operation[A]
    }
  }

  implicit def schema[A](implicit a: Schema[A]): Schema[DynamicEval[A]] = DeriveSchema.gen[DynamicEval[A]]
}
