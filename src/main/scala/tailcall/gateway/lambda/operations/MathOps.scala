package tailcall.gateway.lambda.operations

import tailcall.gateway.lambda.DynamicEval.Math
import tailcall.gateway.lambda._

trait MathOps {
  implicit final class RemoteOps[A](val self: Remote[A]) {
    def >[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[Boolean] =
      Lambda.unsafe
        .attempt(ctx => Math(Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.GreaterThan), tag.any))

    def increment[A1 >: A](implicit tag: Numeric[A1], ctor: Constructor[A1]) = self + Lambda(tag.one)

    def +[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      Lambda.unsafe.attempt(ctx => Math(Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Add), tag.any))

    def -[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] = self + other.negate

    def *[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      Lambda.unsafe
        .attempt(ctx => Math(Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Multiply), tag.any))

    def /[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      Lambda.unsafe
        .attempt(ctx => Math(Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Divide), tag.any))

    def %[A1 >: A](other: Remote[A1])(implicit tag: Numeric[A1]): Remote[A1] =
      Lambda.unsafe
        .attempt(ctx => Math(Math.Binary(self.compile(ctx), other.compile(ctx), Math.Binary.Modulo), tag.any))

    def negate[A1 >: A](implicit tag: Numeric[A1]): Remote[A1] =
      Lambda.unsafe.attempt(ctx => Math(Math.Unary(self.compile(ctx), Math.Unary.Negate), tag.any))

    def debug(message: String): Remote[A] = Lambda.unsafe.attempt(ctx => DynamicEval.Debug(self.compile(ctx), message))
  }
}
