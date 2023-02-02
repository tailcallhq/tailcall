package tailcall.gateway.remote

import zio.schema.DeriveSchema
import zio.schema.Schema
import zio._
import zio.schema.DynamicValue
import zio.schema.meta.MetaSchema

sealed trait DynamicEval

object DynamicEval {
  final case class Literal(value: DynamicValue, meta: MetaSchema) extends DynamicEval

  // TODO: move to Logical
  final case class Diverge(cond: DynamicEval, isTrue: DynamicEval, isFalse: DynamicEval)
      extends DynamicEval
  final case class EqualTo(left: DynamicEval, right: DynamicEval) extends DynamicEval

  final case class Math(operation: Math.Operation) extends DynamicEval
  object Math {
    sealed trait Operation
    final case class Add(left: DynamicEval, right: DynamicEval)      extends Operation
    final case class Negate(value: DynamicEval)                      extends Operation
    final case class Multiply(left: DynamicEval, right: DynamicEval) extends Operation
    final case class Divide(left: DynamicEval, right: DynamicEval)   extends Operation
    final case class Modulo(left: DynamicEval, right: DynamicEval)   extends Operation
  }

  final case class Logical(operation: Logical.Operation) extends DynamicEval
  object Logical {
    sealed trait Operation
    final case class And(left: DynamicEval, right: DynamicEval) extends Operation
    final case class Or(left: DynamicEval, right: DynamicEval)  extends Operation
    final case class Not(value: DynamicEval)                    extends Operation

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

  final case class Apply(f: EvalFunction, arg: DynamicEval)        extends DynamicEval
  final case class Binding private (id: Int)                       extends DynamicEval
  final case class EvalFunction(input: Binding, body: DynamicEval) extends DynamicEval

  implicit val schema: Schema[DynamicEval] = DeriveSchema.gen[DynamicEval]

  def eval(value: DynamicEval): UIO[Any] = ???
}
