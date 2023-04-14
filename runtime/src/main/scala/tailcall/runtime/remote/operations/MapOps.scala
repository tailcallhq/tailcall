package tailcall.runtime.remote.operations

import tailcall.runtime.remote.Remote

trait MapOps {
  implicit final class RemoteMapOps[R, A, B](val self: Remote[R, Map[A, B]]) {
    def get(key: Remote[R, A]): Remote[R, Option[B]]                      = Remote.dict.get(key, self)
    def put(key: Remote[R, A], value: Remote[R, B]): Remote[R, Map[A, B]] = Remote.dict.put(key, value, self)
    def toPair: Remote[R, List[(A, B)]]                                   = self >>> Remote.dict.toPair
  }
}
