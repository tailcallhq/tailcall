package tailcall.runtime.transcoder

import tailcall.runtime.ast.TSchema
import tailcall.runtime.internal.TValid
import zio.json.ast.Json

/**
 * Infers TSchema from a JSON Value.
 */
trait JsonValue2TSchema {
  final def toTSchema(jsonAST: Json): TValid[String, TSchema] =
    jsonAST match {
      case Json.Obj(fields)   => TValid.foreach(fields.toList) { case (name, value) =>
          toTSchema(value).map(TSchema.Field(name, _))
        }.map(TSchema.obj)
      case Json.Arr(elements) => TValid.foreachChunk(elements.map(toTSchema))(identity)
          .map(chunk => chunk.reduce(_ unify _)).map(TSchema.arr)
      case Json.Bool(_)       => TValid.succeed(TSchema.Boolean)
      case Json.Str(_)        => TValid.succeed(TSchema.String)
      case Json.Num(_)        => TValid.succeed(TSchema.Int)
      case Json.Null          => TValid.succeed(TSchema.NULL)
    }

  final def toTSchema(json: String): TValid[String, TSchema] =
    for {
      jsonAST <- TValid.fromEither(Json.decoder.decodeJson(json))
      tSchema <- toTSchema(jsonAST)
    } yield tSchema
}
