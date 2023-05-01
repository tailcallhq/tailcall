package tailcall.runtime

package object lambda {
  type ~>[-A, +B]  = Lambda[A, B]
  type ~>>[-A, +B] = Any ~> A => Any ~> B
}
