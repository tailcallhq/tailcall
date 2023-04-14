package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.{Lambda, ~>}

trait MapOps {
  implicit final class LambdaMapOps[A, K, V](val self: A ~> Map[K, V]) {
    def get(key: A ~> K): A ~> Option[V]                = Lambda.dict.get(key, self)
    def put(key: A ~> K, value: A ~> V): A ~> Map[K, V] = Lambda.dict.put(key, value, self)
    def toPair: A ~> List[(K, V)]                       = self >>> Lambda.dict.toPair
  }
}
