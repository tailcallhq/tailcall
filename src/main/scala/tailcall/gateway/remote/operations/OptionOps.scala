package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}
import zio.schema.Schema

trait OptionOps {
  implicit final class RemoteOptionOps[A](private val self: Remote[Option[A]]) {
    def fold[B](g: Remote[B])(f: Remote[A] => Remote[B]): Remote[B] =
      Remote
        .unsafe
        .attempt(
          DynamicEval.foldOption(self.compile, g.compile, Remote.fromFunction(f).compileAsFunction)
        )

    def map[B](f: Remote[A] => Remote[B])(implicit schema: Schema[Option[B]]): Remote[Option[B]] =
      self.flatMap(a => Remote.fromOption(Some(f(a))))

    def flatMap[B](f: Remote[A] => Remote[Option[B]])(implicit
      schema: Schema[Option[B]]
    ): Remote[Option[B]] = self.fold(Remote(Option.empty[B]))(a => f(a))

    def isNone: Remote[Boolean] = fold(Remote(true))(_ => Remote(false))

    def isSome: Remote[Boolean] = !isNone

    def getOrElse(default: Remote[A]): Remote[A] = fold(default)(identity)

    def getOrDie: Remote[A] = fold(Remote.die(Remote("Value not found")))(identity)
  }
}
