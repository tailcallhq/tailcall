package tailcall.gateway.remote
import zio._

trait RemoteEval {
  def eval[A](remote: Remote[A]): UIO[A]
}

object RemoteEval {
  final case class Default(ctx: Map[Int, Any]) extends RemoteEval {
    import Remote._

    def eval[A](remote: Remote[A]): UIO[A] = ZIO.suspendSucceed {
      remote match {
        case Literal(value)                 => ZIO.succeed(value)
        case Diverge(cond, isTrue, isFalse) => eval(cond)
            .flatMap(i => if (i) eval(isTrue) else eval(isFalse))
        case EqualTo(left, right, tag)      => eval(left).zipWith(eval(right))(tag.equals)
        case Math(operation, tag)           => operation match {
            case Math.Negate(value)         => eval(value).map(tag.negate)
            case Math.Add(left, right)      => eval(left).zipWith(eval(right))(tag.add)
            case Math.Multiply(left, right) => eval(left).zipWith(eval(right))(tag.multiply)
            case Math.Divide(left, right)   => eval(left).zipWith(eval(right))(tag.divide)
            case Math.Modulo(left, right)   => eval(left).zipWith(eval(right))(tag.modulo)
          }
        case Logical(operation)             => operation match {
            case Logical.And(left, right) => eval(left).zipWith(eval(right))(_ && _)
            case Logical.Or(left, right)  => eval(left).zipWith(eval(right))(_ || _)
            case Logical.Not(value)       => eval(value).map(!_)
          }
        case IndexSeqOperations(operation)  => operation match {
            case IndexSeqOperations.Concat(left, right)   => eval(left).zipWith(eval(right))(_ ++ _)
            case IndexSeqOperations.Reverse(seq)          => eval(seq).map(_.reverse)
            case IndexSeqOperations.Filter(seq, cond)     => eval(seq)
                .flatMap(seq => ZIO.filter(seq)(any => eval(cond(Remote(any)))))
            case IndexSeqOperations.FlatMap(_, _)         => ???
            case IndexSeqOperations.Map(_, _)             => ???
            case IndexSeqOperations.Length(seq)           => eval(seq).map(_.length)
            case IndexSeqOperations.IndexOf(seq, element) => eval(seq)
                .zipWith(eval(element))(_ indexOf _)
          }
        case remote: Closure[_]             => remote match {
            case Closure.RemoteFunction(arg, fBody) => new Default(ctx + (arg.id -> arg))
                .eval(fBody.asInstanceOf[Remote[A]])

            case Closure.Ref(id) => ctx.get(id) match {
                case Some(value) => ZIO.succeed(value.asInstanceOf[A])
                case None        => ZIO.dieMessage("No value found for id: " + id)
              }
          }
        case Apply(f, arg)                  => for {
            a <- eval(arg)
            b <- new Default(ctx + (f.ref.id -> a)).eval(f.f)
          } yield b.asInstanceOf[A]
      }
    }
  }

  def make: RemoteEval = Default(Map.empty)
}
