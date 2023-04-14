package tailcall.runtime

import tailcall.runtime.lambda.operations._

package object lambda extends MathOps with DynamicValueOps with BooleanOps with MapOps with OptionOps {
  type ~>[-A, +B]  = Lambda[A, B]
  type ~>>[-A, +B] = Any ~> A => Any ~> B
}
