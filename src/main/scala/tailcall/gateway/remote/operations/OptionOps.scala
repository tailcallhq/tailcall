package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait OptionOps {
  implicit final class RemoteOptionOps[A](private val self: Remote[Option[A]]) {
    def fold[B](g: Remote[B])(f: Remote[A] => Remote[B]): Remote[B] =
      Remote
        .unsafe
        .attempt(
          DynamicEval.foldOption(self.compile, g.compile, Remote.fromFunction(f).compileAsFunction)
        )

    def isNone: Remote[Boolean] = fold(Remote(true))(_ => Remote(false))

    def isSome: Remote[Boolean] = !isNone
  }
}
