package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.OptionOperations
import tailcall.gateway.lambda.{Lambda, Remote}

trait OptionOps {
  implicit final class RemoteOptionOps[A](private val self: Remote[Option[A]]) {
    def fold[B](g: Remote[B])(f: Remote[A] => Remote[B]): Remote[B] =
      Lambda.unsafe.attempt(ctx =>
        OptionOperations(OptionOperations.Fold(self.compile(ctx), g.compile(ctx), Lambda.fromFunction(f).compile(ctx)))
      )

    def map[B](f: Remote[A] => Remote[B]): Remote[Option[B]] = self.flatMap(a => Lambda.fromOption(Some(f(a))))

    def flatMap[B](f: Remote[A] => Remote[Option[B]]): Remote[Option[B]] = self.fold(Lambda.none[B])(a => f(a))

    def isNone: Remote[Boolean] = fold(Lambda(true))(_ => Lambda(false))

    def isSome: Remote[Boolean] = !isNone

    def getOrElse(default: Remote[A]): Remote[A] = fold(default)(identity)

    def getOrDie: Remote[A] = fold(Lambda.die(Lambda("Value not found")))(identity)
  }
}
