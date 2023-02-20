package tailcall.gateway

import tailcall.gateway.lambda.operations._

package object lambda
    extends StringOps
    with SeqOps
    with BooleanOps
    with EitherOps
    with OptionOps
    with ContextOps
    with DynamicValueOps
    with TupleOps
    with MapOps
    with MathOps {

  type ~>[-A, +B] = Lambda[A, B]
  type Remote[+A] = Any ~> A
}
