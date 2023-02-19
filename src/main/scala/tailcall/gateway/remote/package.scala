package tailcall.gateway

import tailcall.gateway.remote.Lambda

package object remote {
  type ~>[-A, +B] = Lambda[A, B]
  type Lazy[+A]   = Any ~> A
}
