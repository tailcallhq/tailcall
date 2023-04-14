package tailcall.runtime.remote.operations

import tailcall.runtime.remote.Remote

trait MapOps {
  implicit final class RemoteMapOps[A, K, V](val self: Remote[A, Map[K, V]]) {
    def get(key: Remote[A, K]): Remote[A, Option[V]]                      = Remote.dict.get(key, self)
    def put(key: Remote[A, K], value: Remote[A, V]): Remote[A, Map[K, V]] = Remote.dict.put(key, value, self)
    def toPair: Remote[A, List[(K, V)]]                                   = self >>> Remote.dict.toPair
  }
}
