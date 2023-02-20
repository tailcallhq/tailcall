package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.EitherOperations
import tailcall.gateway.lambda.{Lambda, Remote}

trait EitherOps {
  implicit final class Extensions[E, A](
    private val self: Remote[Either[E, A]]
  ) {
    def fold[B](
      f: Remote[E] => Remote[B],
      g: Remote[A] => Remote[B]
    ): Remote[B] =
      Lambda
        .unsafe
        .attempt(ctx =>
          EitherOperations(EitherOperations.Fold(
            self.compile(ctx),
            Lambda.fromFunction(f).compile(ctx),
            Lambda.fromFunction(g).compile(ctx)
          ))
        )

    def flatMap[B](f: Remote[A] => Remote[Either[E, B]]): Remote[Either[E, B]] =
      fold[Either[E, B]](e => Lambda.fromEither(Left(e)), a => f(a))

    def map[B](f: Remote[A] => Remote[B]): Remote[Either[E, B]] =
      flatMap(a => Lambda.fromEither(Right(f(a))))

    def toOption: Remote[Option[A]] =
      self.fold[Option[A]](
        _ => Lambda.fromOption(Option.empty),
        a => Lambda.fromOption(Some(a))
      )
  }
}
