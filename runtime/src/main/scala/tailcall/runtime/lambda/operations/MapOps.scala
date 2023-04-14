package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.Lambda

trait MapOps {
  implicit final class LambdaMapOps[A, K, V](val self: Lambda[A, Map[K, V]]) {
    def get(key: Lambda[A, K]): Lambda[A, Option[V]]                      = Lambda.dict.get(key, self)
    def put(key: Lambda[A, K], value: Lambda[A, V]): Lambda[A, Map[K, V]] = Lambda.dict.put(key, value, self)
    def toPair: Lambda[A, List[(K, V)]]                                   = self >>> Lambda.dict.toPair
  }
}
