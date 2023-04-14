package tailcall.runtime

import tailcall.runtime.remote.operations.{BooleanOps, DynamicValueOps, MapOps, MathOps, OptionOps}

package object remote extends MathOps with DynamicValueOps with BooleanOps with MapOps with OptionOps {
  type ~>[-A, +B] = Lambda[A, B]
}
