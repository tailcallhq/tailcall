package tailcall.runtime.remote.operations

import tailcall.runtime.lambda.Lambda
import tailcall.runtime.remote.Remote

trait MapOps {
  implicit final class RemoteMapOps[A, B](val self: Remote[Map[A, B]]) {
    def get(key: Remote[A]): Remote[Option[B]] = Remote(Lambda.dict.get(key.toLambda, self.toLambda))
  }
}
