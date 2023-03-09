package tailcall.runtime

package object lambda {
  type ~>[-A, +B] = Lambda[A, B]
}
