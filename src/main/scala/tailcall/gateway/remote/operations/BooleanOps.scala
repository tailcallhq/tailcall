package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait BooleanOps {
  implicit final class RemoteBooleanOps(val self: Remote[Boolean]) {
    def &&(other: Remote[Boolean]): Remote[Boolean] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.and(self.compile(ctx), other.compile(ctx)))

    def ||(other: Remote[Boolean]): Remote[Boolean] =
      Remote
        .unsafe
        .attempt(ctx => DynamicEval.or(self.compile(ctx), other.compile(ctx)))

    def unary_! : Remote[Boolean] =
      Remote.unsafe.attempt(ctx => DynamicEval.not(self.compile(ctx)))

    def diverge[A](isTrue: Remote[A], isFalse: Remote[A]): Remote[A] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.diverge(
            self.compile(ctx),
            isTrue.compile(ctx),
            isFalse.compile(ctx)
          )
        )
  }
}
