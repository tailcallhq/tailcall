package tailcall.runtime.lambda

import tailcall.runtime.JsonT
import tailcall.runtime.lambda.Expression.{Opt, Sequence, Str, T2Exp}
import zio.Chunk
import zio.schema.{DynamicValue, Schema}

object Syntax {
  implicit class BooleanSyntax[A](self: A ~> Boolean) {
    def &&(other: A ~> Boolean): A ~> Boolean = Lambda.logic.and(self, other)
    def ||(other: A ~> Boolean): A ~> Boolean = Lambda.logic.or(self, other)

    def diverge[B](isTrue: A ~> B, isFalse: A ~> B): A ~> B = Lambda.logic.cond(self)(isTrue, isFalse)

    def unary_! : A ~> Boolean = Lambda.logic.not(self)
  }

  implicit class DynamicValueSyntax[A](self: A ~> DynamicValue) {
    def path(name: String*): A ~> Option[DynamicValue] = self >>> Lambda.dynamic.path(name: _*)

    def pathSeq(name: String*): A ~> Option[DynamicValue] = self >>> Lambda.dynamic.pathSeq(name: _*)

    def toTyped[B](implicit schema: Schema[B]): A ~> Option[B] = self >>> Lambda.dynamic.toTyped[B]

    def toTypedPath[B](name: String*)(implicit schema: Schema[B]): A ~> Option[B] =
      self.path(name: _*).flatMap(_.toTyped[B])
    def transform(jsonT: JsonT): A ~> DynamicValue = self >>> Lambda.dynamic.jsonTransform(jsonT)
  }

  implicit class MapSyntax[A, K, V](self: A ~> Map[K, V]) {
    def get(key: A ~> K): A ~> Option[V] = Lambda.dict.get(key, self)

    def put(key: A ~> K, value: A ~> V): A ~> Map[K, V] = Lambda.dict.put(key, value, self)

    def toPair: A ~> List[(K, V)] = self >>> Lambda.dict.toPair
  }

  implicit class MathSyntax[A, B: Numeric](self: A ~> B) {
    def /(other: A ~> B): A ~> B = Lambda.math.div(self, other)

    def >(other: A ~> B): A ~> Boolean = Lambda.math.gt(self, other)

    def -(other: A ~> B): A ~> B = Lambda.math.sub(self, other)

    def %(other: A ~> B): A ~> B = Lambda.math.mod(self, other)

    def +(other: A ~> B): A ~> B = Lambda.math.add(self, other)

    def *(other: A ~> B): A ~> B = Lambda.math.mul(self, other)

    def unary_- : A ~> B = Lambda.math.neg(self)
  }

  implicit class OptionSyntax[A, B](self: A ~> Option[B]) {
    def flatMap[C](f: B ~>> Option[C])(implicit ev: Schema[C]): A ~> Option[C] =
      self.fold[Option[C]](Lambda(Option.empty[C]), f(_))

    def flatten[C](implicit ev: A ~> Option[B] <:< (A ~> Option[Option[C]]), schema: Schema[C]): Lambda[A, Option[C]] =
      ev(self).flatMap(identity(_))

    def fold[C](ifNone: A ~> C, ifSome: B ~>> C): A ~> C =
      Lambda.option.fold[A, B, C](self, ifNone, Lambda.fromFunction(ifSome))

    def getOrElse[B1 >: B](default: A ~> B1): A ~> B1 = Lambda.option.fold(self, default, Lambda.identity[B1])

    def isNone: A ~> Boolean = self >>> Lambda.option.isNone

    def isSome: A ~> Boolean = self >>> Lambda.option.isSome

    def map[C](f: B ~>> C)(implicit ev: Schema[C]): A ~> Option[C] =
      self.fold[Option[C]](Lambda(Option.empty[C]), a => Lambda.option(Option(f(a))))

    def toSeq: A ~> Seq[B] = Lambda.unsafe.attempt(ctx => Opt(Opt.ToSeq(self.compile(ctx))))
  }

  implicit class Tuple2Syntax[A, A1, A2](self: A ~> (A1, A2)) {
    def _1: A ~> A1 = Lambda.unsafe.attempt(ctx => Expression.T2Exp(self.compile(ctx), T2Exp._1))
    def _2: A ~> A2 = Lambda.unsafe.attempt(ctx => Expression.T2Exp(self.compile(ctx), T2Exp._2))
  }

  implicit class SeqSyntax[A, B](self: A ~> Seq[B]) {
    def flatMap[C](bc: B ~>> Seq[C]): A ~> Seq[C] =
      Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.FlatMap(Lambda.fromFunction(bc).compile(ctx))))

    def flatten[C](implicit ev: Seq[B] <:< Seq[Seq[C]]): A ~> Seq[C] = self.widen[Seq[Seq[C]]].flatMap(identity(_))

    def groupBy[K](f: B ~>> K): A ~> Map[K, Seq[B]] =
      Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.GroupBy(Lambda.fromFunction(f).compile(ctx))))

    def map[C](bc: B ~>> C): A ~> Seq[C] =
      Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.Map(Lambda.fromFunction(bc).compile(ctx))))

    def mkString: A ~> String = Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.MakeString))

    def toChunk: A ~> Chunk[B] = Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.ToChunk))

    def head: A ~> Option[B] = Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.Head))
  }

  implicit class StringSyntax[A](self: A ~> String) {
    def ++(other: A ~> String): A ~> String =
      Lambda.unsafe.attempt(ctx => Str(self.compile(ctx), Str.Concat(other.compile(ctx))))
  }
}
