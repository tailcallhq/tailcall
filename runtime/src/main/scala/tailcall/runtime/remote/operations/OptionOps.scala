package tailcall.runtime.remote.operations

import tailcall.runtime.remote.Remote
import zio.schema.Schema

trait OptionOps {
  implicit final class RemoteOptionOps[A, B](private val self: Remote[A, Option[B]]) {
    def isSome: Remote[A, Boolean] = self >>> Remote.option.isSome

    def isNone: Remote[A, Boolean] = self >>> Remote.option.isNone

    def fold[C](ifNone: Remote[A, C], ifSome: Remote[Any, B] => Remote[Any, C]): Remote[A, C] =
      Remote.option.fold[A, B, C](self, ifNone, Remote.fromFunction(ifSome))

    def getOrElse[B1 >: B](default: Remote[A, B1]): Remote[A, B1] =
      Remote.option.fold(self, default, Remote.identity[B1])

    def flatMap[C](f: Remote[Any, B] => Remote[Any, Option[C]])(implicit ev: Schema[C]): Remote[A, Option[C]] =
      self.fold[Option[C]](Remote(Option.empty[C]), f(_))

    def flatten[C](implicit
      ev: Remote[A, Option[B]] <:< Remote[A, Option[Option[C]]],
      schema: Schema[C],
    ): Remote[A, Option[C]] = ev(self).flatMap(identity(_))

    def map[C](f: Remote[Any, B] => Remote[Any, C])(implicit ev: Schema[C]): Remote[A, Option[C]] =
      self.fold[Option[C]](Remote(Option.empty[C]), a => Remote.option(Option(f(a))))
  }
}
