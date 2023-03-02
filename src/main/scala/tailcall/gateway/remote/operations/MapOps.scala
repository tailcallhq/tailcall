package tailcall.gateway.remote.operations

import tailcall.gateway.lambda.Lambda
import tailcall.gateway.remote.Remote

trait MapOps:
  extension [A, B](self: Remote[Map[A, B]])
    def get(key: Remote[A]): Remote[Option[B]] = Remote(Lambda.dict.get(key.toLambda, self.toLambda))
