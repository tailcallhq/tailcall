package tailcall.gateway.remote.operations

import tailcall.gateway.remote.Remote
import tailcall.gateway.lambda.Lambda

trait OptionOps {
  implicit final class RemoteOptionOps[A](private val self: Remote[Option[A]]) {
    def isSome: Remote[Boolean] = Remote(self.toLambda >>> Lambda.option.isSome)

    def isNone: Remote[Boolean] = Remote(self.toLambda >>> Lambda.option.isNone)

    def fold[B](ifNone: Remote[B], ifSome: Remote[A] => Remote[B]): Remote[B] =
      Remote(Lambda.option.fold[Any, A, B](self.toLambda, ifNone.toLambda, Remote.fromRemoteFunction(ifSome)))

    def getOrElse[B >: A](default: Remote[B]): Remote[B] =
      Remote(Lambda.option.fold(self.toLambda, default.toLambda, Remote.fromRemoteFunction[B, B](i => i)))

    def getOrDie: Remote[A] = getOrElse(Remote.die("Failed to get value from None"))
  }
}
