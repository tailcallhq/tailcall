package tailcall.gateway

package object remote {
  type ~>[-A, +B] = Lambda[A, B]
  type Lazy[+A]   = Any ~> A
  type Remote[+A] = Any ~> A
}
