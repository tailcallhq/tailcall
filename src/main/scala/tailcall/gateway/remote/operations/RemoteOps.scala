package tailcall.gateway.remote.operations

import tailcall.gateway.remote.DynamicEval.{EqualTo, Math}
import tailcall.gateway.remote.{
  CompilationContext,
  Constructor,
  DynamicEval,
  Equatable,
  LambdaRuntime,
  Numeric,
  Remote
}
import zio.ZIO
import zio.schema.{DynamicValue, Schema}

trait RemoteOps {
  self =>
  implicit final class RemoteOps[A](val self: Remote[A]) {

    import Remote.unsafe.attempt

    def compile(context: CompilationContext): DynamicEval

    def =:=[A1 >: A](
      other: Remote[A1]
    )(implicit tag: Equatable[A1]): Remote[Boolean] =
      attempt(ctx => EqualTo(self.compile(ctx), other.compile(ctx), tag.any))

    def >[A1 >: A](
      other: Remote[A1]
    )(implicit tag: Numeric[A1]): Remote[Boolean] =
      attempt(ctx =>
        Math(
          Math.Binary(
            self.compile(ctx),
            other.compile(ctx),
            Math.Binary.GreaterThan
          ),
          tag.any
        )
      )

    def increment[A1 >: A](implicit tag: Numeric[A1], ctor: Constructor[A1]) =
      self + Remote(tag.one)

    def +[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      attempt(ctx =>
        Math(
          Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Add),
          tag.any
        )
      )

    def -[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      self + other.negate

    def *[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      attempt(ctx =>
        Math(
          Math.Binary(
            self.compile(ctx),
            other.compile(ctx),
            Math.Binary.Multiply
          ),
          tag.any
        )
      )

    def /[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      attempt(ctx =>
        Math(
          Math
            .Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Divide),
          tag.any
        )
      )

    def %[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      attempt(ctx =>
        Math(
          Math
            .Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Modulo),
          tag.any
        )
      )

    def negate[A1 >: A](implicit tag: Numeric[A1]): Remote[A1] =
      attempt(ctx =>
        Math(Math.Unary(self.compile(ctx), Math.Unary.Negate), tag.any)
      )

    def debug(message: String): Remote[A] =
      attempt(ctx => DynamicEval.Debug(self.compile(ctx), message))

    def toDynamicValue[A1 >: A](implicit ev: Schema[A1]): Remote[DynamicValue] =
      ???

    def flatten[B](implicit ev: Remote[A] <:< Remote[Remote[B]]): Remote[B] =
      Remote.flatten(self)

    def evaluate: ZIO[LambdaRuntime, Throwable, A] =
      LambdaRuntime.evaluate(self)(())
  }
}
