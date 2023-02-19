package tailcall.gateway.remote

import tailcall.gateway.ast.{Context, Orc}
import zio.Chunk
import zio.schema.{DynamicValue, Schema, StandardType}

final case class Constructor[A](schema: Schema[A]) {
  def any: Constructor[Any] = this.asInstanceOf[Constructor[Any]]
}

object Constructor {
  implicit def primitive[A](implicit s: StandardType[A]): Constructor[A] =
    Constructor(Schema.primitive(s))

  implicit def seq[A](implicit a: Constructor[A]): Constructor[Seq[A]] = {
    implicit val schemaA: Schema[A] = a.schema
    Constructor(Schema[Chunk[A]].transform(_.toSeq, Chunk.fromIterable))
  }

  implicit val dynamicValue: Constructor[DynamicValue] =
    Constructor(Schema[DynamicValue])

  implicit val context: Constructor[Context] = Constructor(Schema[Context])

  implicit def tuple2[A1, A2](implicit
    ctor1: Constructor[A1],
    ctor2: Constructor[A2]
  ): Constructor[(A1, A2)] = {
    implicit val schema1: Schema[A1] = ctor1.schema
    implicit val schema2: Schema[A2] = ctor2.schema
    Constructor(Schema[(A1, A2)])
  }
  implicit def map[A, B](implicit
    ctorA: Constructor[A],
    ctorB: Constructor[B]
  ): Constructor[Map[A, B]] = {
    implicit val schemaA: Schema[A] = ctorA.schema
    implicit val schemaB: Schema[B] = ctorB.schema
    Constructor(Schema[Map[A, B]])
  }

  implicit def orc: Constructor[Orc] = Constructor(Schema[Orc])

  implicit def remote[A]: Constructor[Remote[A]] =
    Constructor(Schema[Remote[A]])
  implicit def option[A](implicit
    ctor: Constructor[A]
  ): Constructor[Option[A]] = {
    implicit val schema: Schema[A] = ctor.schema
    Constructor(Schema[Option[A]])
  }

  implicit def lambda[A, B]: Constructor[A ~> B] = Constructor(Schema[A ~> B])
}
