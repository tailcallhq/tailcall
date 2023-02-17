package tailcall.gateway.remote

import tailcall.gateway.ast.Context
import tailcall.gateway.ast.Orc.OExit
import zio.Chunk
import zio.schema.{DynamicValue, Schema, StandardType}

final case class Constructor[A](schema: Schema[A])

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

  implicit val oExit: Constructor[OExit] = Constructor(Schema[OExit])

  implicit def tuple2[A1, A2](implicit
    ctor1: Constructor[A1],
    ctor2: Constructor[A2]
  ): Constructor[(A1, A2)] = {
    implicit val schema1: Schema[A1] = ctor1.schema
    implicit val schema2: Schema[A2] = ctor2.schema
    Constructor(Schema[(A1, A2)])
  }
}
