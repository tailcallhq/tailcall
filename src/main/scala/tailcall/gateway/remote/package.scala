package tailcall.gateway

package object remote {
  type ~>[-A, +B] = Lambda[A, B]
}
