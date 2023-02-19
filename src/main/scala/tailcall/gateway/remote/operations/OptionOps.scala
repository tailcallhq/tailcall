package tailcall.gateway.remote.operations

import tailcall.gateway.remote.DynamicEval.OptionOperations
import tailcall.gateway.remote.Remote

trait OptionOps {
  implicit final class RemoteOptionOps[A](private val self: Remote[Option[A]]) {
    def fold[B](g: Remote[B])(f: Remote[A] => Remote[B]): Remote[B] =
      Remote
        .unsafe
        .attempt(ctx =>
          OptionOperations(OptionOperations.Fold(
            self.compile(ctx),
            g.compile(ctx),
            Remote.fromFunction(f).compile(ctx)
          ))
        )

    def map[B](f: Remote[A] => Remote[B]): Remote[Option[B]] =
      self.flatMap(a => Remote.fromOption(Some(f(a))))

    def flatMap[B](f: Remote[A] => Remote[Option[B]]): Remote[Option[B]] =
      self.fold(Remote.none[B])(a => f(a))

    def isNone: Remote[Boolean] = fold(Remote(true))(_ => Remote(false))

    def isSome: Remote[Boolean] = !isNone

    def getOrElse(default: Remote[A]): Remote[A] = fold(default)(identity)

    def getOrDie: Remote[A] =
      fold(Remote.die(Remote("Value not found")))(identity)
  }
}
