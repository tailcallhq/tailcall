package tailcall.gateway.remote
import tailcall.gateway.remote.Remote.IndexSeqOperations.{Filter, FlatMap, IndexOf, Length, Reverse}
import tailcall.gateway.remote.Remote.Logical.{And, Not, Or}
import tailcall.gateway.remote.Remote.StringOperations.Concat
trait Remote2Eval {
  import tailcall.gateway.remote.Remote._

  def fromRemote[A](remote: Remote[A]): DynamicEval = remote match {
    case Binding(id)                    => DynamicEval.Binding(id)
    case Literal(value, schema)         => DynamicEval.Literal(schema.toDynamic(value), schema.ast)
    case Math(operation, _)             => DynamicEval.Math(operation match {
        case Remote.Math.Divide(left, right)   => DynamicEval.Math
            .Divide(fromRemote(left), fromRemote(right))
        case Remote.Math.Add(left, right)      => DynamicEval.Math
            .Add(fromRemote(left), fromRemote(right))
        case Remote.Math.Negate(value)         => DynamicEval.Math.Negate(fromRemote(value))
        case Remote.Math.Multiply(left, right) => DynamicEval.Math
            .Multiply(fromRemote(left), fromRemote(right))
        case Remote.Math.Modulo(left, right)   => DynamicEval.Math
            .Modulo(fromRemote(left), fromRemote(right))
      })
    case Diverge(cond, isTrue, isFalse) => DynamicEval
        .Diverge(fromRemote(cond), fromRemote(isTrue), fromRemote(isFalse))
    case StringOperations(operation)    => DynamicEval.StringOperations(operation match {
        case Concat(left, right) => DynamicEval.StringOperations
            .Concat(fromRemote(left), fromRemote(right))
      })
    case IndexSeqOperations(operation)  => DynamicEval.IndexSeqOperations(operation match {
        case IndexOf(seq, element)        => DynamicEval.IndexSeqOperations
            .IndexOf(fromRemote(seq), fromRemote(element))
        case Filter(seq, condition, _)    => DynamicEval.IndexSeqOperations
            .Filter(fromRemote(seq), fromRemote(condition))
        case IndexSeqOperations.Map(_, _) => ???
        case Length(seq)                  => DynamicEval.IndexSeqOperations.Length(fromRemote(seq))
        case FlatMap(_, _)                => ???
        case IndexSeqOperations.Concat(left, right) => DynamicEval.IndexSeqOperations
            .Concat(fromRemote(left), fromRemote(right))
        case Reverse(seq) => DynamicEval.IndexSeqOperations.Reverse(fromRemote(seq))
      })
    case Logical(operation)             => DynamicEval.Logical(operation match {
        case And(left, right) => DynamicEval.Logical.And(fromRemote(left), fromRemote(right))
        case Not(value)       => DynamicEval.Logical.Not(fromRemote(value))
        case Or(left, right)  => DynamicEval.Logical.Or(fromRemote(left), fromRemote(right))
      })
    case Apply(f, arg)                  => DynamicEval
        .Apply(fromRemote(f).asInstanceOf[DynamicEval.EvalFunction], fromRemote(arg))
    case EqualTo(left, right, _)        => DynamicEval.EqualTo(fromRemote(left), fromRemote(right))
    case RemoteFunction(input, body)    => DynamicEval
        .EvalFunction(fromRemote(input).asInstanceOf[DynamicEval.Binding], fromRemote(body))
  }
}
