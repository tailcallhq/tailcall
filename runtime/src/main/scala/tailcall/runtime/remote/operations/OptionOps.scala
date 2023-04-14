package tailcall.runtime.remote.operations

import tailcall.runtime.remote.{Lambda, Remote}
import zio.schema.Schema

trait OptionOps {
  implicit final class RemoteOptionOps[A](private val self: Remote[Option[A]]) {
    def isSome: Remote[Boolean] = Remote(self.toLambda >>> Lambda.option.isSome)

    def isNone: Remote[Boolean] = Remote(self.toLambda >>> Lambda.option.isNone)

    def fold[B](ifNone: Remote[B], ifSome: Remote[A] => Remote[B]): Remote[B] =
      Remote(Lambda.option.fold[Any, A, B](self.toLambda, ifNone.toLambda, Remote.toLambda(ifSome)))

    def getOrElse[B >: A](default: Remote[B]): Remote[B] =
      Remote(Lambda.option.fold(self.toLambda, default.toLambda, Remote.toLambda[B, B](i => i)))

    def getOrDie: Remote[A] = getOrElse(Remote.die("Failed to get value from None"))

    def flatMap[B](f: Remote[A] => Remote[Option[B]])(implicit ev: Schema[B]): Remote[Option[B]] =
      self.fold[Option[B]](Remote(Option.empty[B]), f(_))

    def flatten[B](implicit ev: Remote[Option[A]] <:< Remote[Option[Option[B]]], schema: Schema[B]): Remote[Option[B]] =
      ev(self).flatMap(identity(_))

    def map[B](f: Remote[A] => Remote[B])(implicit ev: Schema[B]): Remote[Option[B]] =
      self.fold[Option[B]](Remote(Option.empty[B]), a => Remote.fromOption(Option(f(a))))
  }
}
