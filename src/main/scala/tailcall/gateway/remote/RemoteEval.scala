package tailcall.gateway.remote
import zio._

trait RemoteEval {
  import Remote._

  private[tailcall] def eval[A](remote: Remote[A]): UIO[A] = remote match {
    case Literal(value)                 => ZIO.succeed(value)
    case Diverge(cond, isTrue, isFalse) => cond.eval
        .flatMap(i => if (i) isTrue.eval else isFalse.eval)
    case EqualTo(left, right, tag)      => left.eval.zipWith(right.eval)(tag.equals)
    case Math(operation, tag)           => operation match {
        case Math.Negate(value)         => value.eval.map(tag.negate)
        case Math.Add(left, right)      => left.eval.zipWith(right.eval)(tag.add)
        case Math.Multiply(left, right) => left.eval.zipWith(right.eval)(tag.multiply)
        case Math.Divide(left, right)   => left.eval.zipWith(right.eval)(tag.divide)
        case Math.Modulo(left, right)   => left.eval.zipWith(right.eval)(tag.modulo)
      }
    case Logical(operation)             => operation match {
        case Logical.And(left, right) => left.eval.zipWith(right.eval)(_ && _)
        case Logical.Or(left, right)  => left.eval.zipWith(right.eval)(_ || _)
        case Logical.Not(value)       => value.eval.map(!_)
      }
    case IndexSeqOperations(operation)  => operation match {
        case IndexSeqOperations.Concat(left, right)     => left.eval.zipWith(right.eval)(_ ++ _)
        case IndexSeqOperations.Reverse(seq)            => seq.eval.map(_.reverse)
        case IndexSeqOperations.Filter(seq, condition)  => ???
        case IndexSeqOperations.FlatMap(seq, operation) => ???
        case IndexSeqOperations.Map(seq, operation)     => ???
        case IndexSeqOperations.Length(seq)             => seq.eval.map(_.length)
        case IndexSeqOperations.IndexOf(seq, element) => seq.eval.zipWith(element.eval)(_ indexOf _)
      }
  }
}
