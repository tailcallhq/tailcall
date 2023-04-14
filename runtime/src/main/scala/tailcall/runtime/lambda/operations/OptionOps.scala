package tailcall.runtime.lambda.operations

import tailcall.runtime.lambda.Lambda
import zio.schema.Schema

trait OptionOps {
  implicit final class LambdaOptionOps[A, B](private val self: Lambda[A, Option[B]]) {
    def isSome: Lambda[A, Boolean] = self >>> Lambda.option.isSome

    def isNone: Lambda[A, Boolean] = self >>> Lambda.option.isNone

    def fold[C](ifNone: Lambda[A, C], ifSome: Lambda[Any, B] => Lambda[Any, C]): Lambda[A, C] =
      Lambda.option.fold[A, B, C](self, ifNone, Lambda.fromFunction(ifSome))

    def getOrElse[B1 >: B](default: Lambda[A, B1]): Lambda[A, B1] =
      Lambda.option.fold(self, default, Lambda.identity[B1])

    def flatMap[C](f: Lambda[Any, B] => Lambda[Any, Option[C]])(implicit ev: Schema[C]): Lambda[A, Option[C]] =
      self.fold[Option[C]](Lambda(Option.empty[C]), f(_))

    def flatten[C](implicit
      ev: Lambda[A, Option[B]] <:< Lambda[A, Option[Option[C]]],
      schema: Schema[C],
    ): Lambda[A, Option[C]] = ev(self).flatMap(identity(_))

    def map[C](f: Lambda[Any, B] => Lambda[Any, C])(implicit ev: Schema[C]): Lambda[A, Option[C]] =
      self.fold[Option[C]](Lambda(Option.empty[C]), a => Lambda.option(Option(f(a))))
  }
}
