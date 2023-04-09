package tailcall.runtime.remote.operations

import tailcall.runtime.lambda.Lambda
import tailcall.runtime.remote.Remote

trait MapOps {
  implicit final class RemoteMapOps[A, B](val self: Remote[Map[A, B]]) {
    def get(key: Remote[A]): Remote[Option[B]]                   = Remote(Lambda.dict.get(key.toLambda, self.toLambda))
    def put(key: Remote[A], value: Remote[B]): Remote[Map[A, B]] =
      Remote(Lambda.dict.put(key.toLambda, value.toLambda, self.toLambda))
    def toPair: Remote[List[(A, B)]]                             = Remote(self.toLambda >>> Lambda.dict.toPair)
  }
}
