package tailcall.gateway

package object lambda {
  type ~>[-A, +B] = Lambda[A, B]
  type Lazy[+A]   = Any ~> A
}
