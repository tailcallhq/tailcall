package tailcall.gateway

import tailcall.gateway.remote.operations._

package object remote
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
