package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait EitherOps {
  implicit final class RemoteEitherOps[E, A](private val self: Remote[Either[E, A]]) {
    def fold[B](f: Remote[E] => Remote[B], g: Remote[A] => Remote[B]): Remote[B] = Remote.unsafe
      .attempt(DynamicEval.foldEither(
        self.compile,
        Remote.fromFunction(f).compileAsFunction,
        Remote.fromFunction(g).compileAsFunction
      ))
  }
}
