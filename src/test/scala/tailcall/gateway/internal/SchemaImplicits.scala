package tailcall.gateway.internal

import zio.Chunk
import zio.schema.Schema

trait SchemaImplicits:
  implicit def seqSchema[A: Schema]: Schema[Seq[A]] = Schema.chunk[A].transform(_.toSeq, Chunk.from(_))
