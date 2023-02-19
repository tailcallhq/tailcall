package tailcall.gateway.remote.operations

import tailcall.gateway.remote.DynamicEval.EitherOperations
import tailcall.gateway.remote.Remote

trait EitherOps {
  implicit final class RemoteEitherOps[E, A](
    private val self: Remote[Either[E, A]]
  ) {
    def fold[B](
      f: Remote[E] => Remote[B],
      g: Remote[A] => Remote[B]
    ): Remote[B] =
      Remote
        .unsafe
        .attempt(ctx =>
          EitherOperations(EitherOperations.Fold(
            self.compile(ctx),
            Remote.fromFunction(f).compile(ctx),
            Remote.fromFunction(g).compile(ctx)
          ))
        )

    def flatMap[B](f: Remote[A] => Remote[Either[E, B]]): Remote[Either[E, B]] =
      fold[Either[E, B]](e => Remote.fromEither(Left(e)), a => f(a))

    def map[B](f: Remote[A] => Remote[B]): Remote[Either[E, B]] =
      flatMap(a => Remote.fromEither(Right(f(a))))

    def toOption: Remote[Option[A]] =
      self.fold[Option[A]](
        _ => Remote.fromOption(Option.empty),
        a => Remote.fromOption(Some(a))
      )
  }
}
