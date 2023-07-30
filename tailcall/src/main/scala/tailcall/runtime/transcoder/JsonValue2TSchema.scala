package tailcall.runtime.transcoder

import tailcall.runtime.SchemaUnifier
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.TSchema
import zio.json.ast.Json

/**
 * Infers TSchema from a JSON Value.
 */
trait JsonValue2TSchema {

  final def toTSchema(json: String): TValid[String, TSchema] =
    for {
      jsonAST <- TValid.fromEither(Json.decoder.decodeJson(json))
      tSchema <- toTSchema(jsonAST)
    } yield tSchema

  final def toTSchema(jsonAST: Json): TValid[String, TSchema] = {
    jsonAST match {
      case Json.Obj(fields) => for {
          schema <- TValid.foreachChunk(fields) { case (name, value) =>
            val sName = if (name.forall(_.isDigit)) s"_$name" else name
            toTSchema(value).map((sName, _))
          }.map(fields => TSchema.obj(fields.toMap))
        } yield schema

      case Json.Arr(element) => for {
          chunk  <- TValid.foreachChunk(element)(json => toTSchema(json))
          schema <- SchemaUnifier.unify(chunk.toList).map(_.getOrElse(TSchema.str))
        } yield schema.arr

      case Json.Bool(_) => TValid.succeed(TSchema.Bool)
      case Json.Str(_)  => TValid.succeed(TSchema.Str)
      case Json.Num(_)  => TValid.succeed(TSchema.Num)
      case Json.Null    => TValid.succeed(TSchema.obj())
    }
  }

}
