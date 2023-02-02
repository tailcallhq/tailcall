package tailcall.gateway.remote

object UnsafeEvaluator {
  import DynamicEval._

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
        case Logical.Binary(left, right, operation) =>
          val leftValue  = evaluateTyped[Boolean](left)
          val rightValue = evaluateTyped[Boolean](right)
          operation match {
            case Logical.Binary.And => leftValue && rightValue
            case Logical.Binary.Or  => leftValue || rightValue
          }
        case Logical.Unary(value, operation)        =>
          val a = evaluateTyped[Boolean](value)
          operation match {
            case Logical.Unary.Not                      => !a
            case Logical.Unary.Diverge(isTrue, isFalse) =>
              if (a) evaluate(isTrue) else evaluate(isFalse)
          }
      }
    case StringOperations(operation)   => operation match {
        case StringOperations.Concat(left, right) =>
          evaluateTyped[String](left) ++ evaluateTyped[String](right)
      }
    case IndexSeqOperations(operation) => ???
    case Apply(f, arg)                 => ???
    case Binding(id)                   => ???
    case EvalFunction(input, body)     => ???
  }

}
