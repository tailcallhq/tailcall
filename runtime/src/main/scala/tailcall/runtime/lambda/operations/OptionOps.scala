package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.{Lambda, ~>}
import zio.schema.Schema

trait OptionOps {
  implicit final class LambdaOptionOps[A, B](private val self: A ~> Option[B]) {
    def isSome: A ~> Boolean = self >>> Lambda.option.isSome

    def isNone: A ~> Boolean = self >>> Lambda.option.isNone

    def fold[C](ifNone: A ~> C, ifSome: Any ~> B => Any ~> C): A ~> C =
      Lambda.option.fold[A, B, C](self, ifNone, Lambda.fromFunction(ifSome))

    def getOrElse[B1 >: B](default: A ~> B1): A ~> B1 = Lambda.option.fold(self, default, Lambda.identity[B1])

    def flatMap[C](f: Any ~> B => Any ~> Option[C])(implicit ev: Schema[C]): A ~> Option[C] =
      self.fold[Option[C]](Lambda(Option.empty[C]), f(_))

    def flatten[C](implicit ev: A ~> Option[B] <:< (A ~> Option[Option[C]]), schema: Schema[C]): Lambda[A, Option[C]] =
      ev(self).flatMap(identity(_))

    def map[C](f: Any ~> B => Any ~> C)(implicit ev: Schema[C]): A ~> Option[C] =
      self.fold[Option[C]](Lambda(Option.empty[C]), a => Lambda.option(Option(f(a))))
  }
}
