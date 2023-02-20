package tailcall.gateway.remote

import tailcall.gateway.lambda.{Constructor, Lambda, LambdaRuntime, ~>}
import zio.ZIO

/**
 * Remote[A] Allows for any arbitrary computation that can
 * be serialized and when evaluated produces a result of
 * type A. This is the lowest level primitive that’s
 * extremely powerful. We use this inside the compiler to
 * convert the composition logic into some form of a Remote.
 */
final case class Remote[+A](toLambda: Any ~> A) {
  def evaluate: ZIO[LambdaRuntime, Throwable, A] = toLambda.evaluate {}
}

object Remote {
  def literal[A](a: A)(implicit c: Constructor[A]): Remote[A] =
    Remote(Lambda(a))
}
