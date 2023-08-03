package tailcall.runtime.internal

import tailcall.runtime.transcoder.Transcoder
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}

object JsonSchema {
  def schema: Schema[Json] =
    Schema[DynamicValue].transformOrFail[Json](
      Transcoder.toJson(_).toEither.fold(chunk => Left(chunk.mkString(", ")), result => Right(result)),
      Transcoder.toDynamicValue(_).toEither.fold(chunk => Left(chunk.mkString(", ")), result => Right(result)),
    )
}
