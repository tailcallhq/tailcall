package tailcall.runtime.remote.operations

import tailcall.runtime.remote.Remote
import zio.schema.Schema

trait OptionOps {
  implicit final class RemoteOptionOps[R, A](private val self: Remote[R, Option[A]]) {
    def isSome: Remote[R, Boolean] = self >>> Remote.option.isSome

    def isNone: Remote[R, Boolean] = self >>> Remote.option.isNone

    def fold[B](ifNone: Remote[R, B], ifSome: Remote[Any, A] => Remote[Any, B]): Remote[R, B] =
      Remote.option.fold[R, A, B](self, ifNone, Remote.fromFunction(ifSome))

    def getOrElse[B >: A](default: Remote[R, B]): Remote[R, B] = Remote.option.fold(self, default, Remote.identity[B])

    def flatMap[B](f: Remote[Any, A] => Remote[Any, Option[B]])(implicit ev: Schema[B]): Remote[R, Option[B]] =
      self.fold[Option[B]](Remote(Option.empty[B]), f(_))

    def flatten[B](implicit
      ev: Remote[R, Option[A]] <:< Remote[R, Option[Option[B]]],
      schema: Schema[B],
    ): Remote[R, Option[B]] = ev(self).flatMap(identity(_))

    def map[B](f: Remote[Any, A] => Remote[Any, B])(implicit ev: Schema[B]): Remote[R, Option[B]] =
      self.fold[Option[B]](Remote(Option.empty[B]), a => Remote.option(Option(f(a))))
  }
}
