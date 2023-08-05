package tailcall.runtime.lambda

import tailcall.runtime.JsonT
import tailcall.runtime.lambda.Expression.Sequence
import zio.Chunk
import zio.schema.{DynamicValue, Schema}

object Syntax {

  implicit class DynamicValueSyntax[A](self: A ~> DynamicValue) {
    def path(name: String*): A ~> Option[DynamicValue] = self >>> Lambda.dynamic.path(name: _*)

    def pathSeq(name: String*): A ~> Option[DynamicValue] = self >>> Lambda.dynamic.pathSeq(name: _*)

    def toTyped[B](implicit schema: Schema[B]): A ~> Option[B] = self >>> Lambda.dynamic.toTyped[B]

    def transform(jsonT: JsonT): A ~> DynamicValue = self >>> Lambda.dynamic.jsonTransform(jsonT)
  }

  implicit class MapSyntax[A, K, V](self: A ~> Map[K, V]) {
    def get(key: A ~> K): A ~> Option[V] = Lambda.dict.get(key, self)

    def put(key: A ~> K, value: A ~> V): A ~> Map[K, V] = Lambda.dict.put(key, value, self)

    def toPair: A ~> List[(K, V)] = self >>> Lambda.dict.toPair
  }

  implicit class OptionSyntax[A, B](self: A ~> Option[B]) {
    def flatMap[C](f: B ~>> Option[C])(implicit ev: Schema[C]): A ~> Option[C] =
      self.fold[Option[C]](Lambda(Option.empty[C]), f(_))

    def fold[C](ifNone: A ~> C, ifSome: B ~>> C): A ~> C =
      Lambda.option.fold[A, B, C](self, ifNone, Lambda.fromFunction(ifSome))

    def getOrElse[B1 >: B](default: A ~> B1): A ~> B1 = Lambda.option.fold(self, default, Lambda.identity[B1])

    def map[C](f: B ~>> C)(implicit ev: Schema[C]): A ~> Option[C] =
      self.fold[Option[C]](Lambda(Option.empty[C]), a => Lambda.option(Option(f(a))))

  }

  implicit class SeqSyntax[A, B](self: A ~> Seq[B]) {
    def flatMap[C](bc: B ~>> Seq[C]): A ~> Seq[C] =
      Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.FlatMap(Lambda.fromFunction(bc).compile(ctx))))


    def groupBy[K](f: B ~>> K): A ~> Map[K, Seq[B]] =
      Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.GroupBy(Lambda.fromFunction(f).compile(ctx))))

    def toChunk: A ~> Chunk[B] = Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.ToChunk))

    def head: A ~> Option[B] = Lambda.unsafe.attempt(ctx => Sequence(self.compile(ctx), Sequence.Head))
  }

}
