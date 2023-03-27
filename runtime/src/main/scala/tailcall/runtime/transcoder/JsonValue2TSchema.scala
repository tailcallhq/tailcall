package tailcall.runtime.transcoder

import tailcall.runtime.ast.TSchema
import tailcall.runtime.internal.TValid
import zio.json.ast.Json

/**
 * Infers TSchema from a JSON Value.
 */
trait JsonValue2TSchema {

  /**
   * Unifies two schemas into a single schema that is a
   * supertype of both. The unify function is different from
   * the union function because it is not just combining two
   * types into a single Union type. Instead, it is creating
   * a new schema that includes all the properties of both
   * input schemas. This is done to reduce unnecessary
   * unions.
   */
  private def unify(a: TSchema, b: TSchema): TValid[String, TSchema] =
    (a, b) match {
      case (TSchema.Int, TSchema.Int)                   => TValid.succeed(TSchema.Int)
      case (TSchema.String, TSchema.String)             => TValid.succeed(TSchema.String)
      case (TSchema.Boolean, TSchema.Boolean)           => TValid.succeed(TSchema.Boolean)
      case (TSchema.Obj(fields1), TSchema.Obj(fields2)) =>
        val field1Map: Map[String, TSchema] = fields1.map(f => f.name -> f.schema).toMap
        val field2Map: Map[String, TSchema] = fields2.map(f => f.name -> f.schema).toMap

        for {
          fields <- TValid.foreachIterable(field1Map.keys ++ field2Map.keys) { key =>
            val fieldDesc = (field1Map.get(key), field2Map.get(key)) match {
              case (Some(s1), Some(s2)) => unify(s1, s2)
              case (Some(s1), None)     => TValid.succeed(s1.opt)
              case (None, Some(s2))     => TValid.succeed(s2.opt)
              case (None, None)         => TValid.fail(s"Key ${key} should be present in one of the maps")
            }

            fieldDesc.map(TSchema.Field(key, _))
          }
        } yield TSchema.obj(fields.toList)

      case (TSchema.Arr(item1), TSchema.Arr(item2)) => unify(item1, item2).map(TSchema.arr(_))
      case _                                        => TValid.fail(s"Cannot identify generate schema for ${a} and ${b}")
    }

  private def unify(list: List[TSchema]): TValid[String, TSchema] =
    list match {
      case Nil          => TValid.fail("Cannot unify an empty list")
      case head :: Nil  => TValid.succeed(head)
      case head :: tail => unify(tail).flatMap(unify(head, _))
    }

  final def toTSchema(jsonAST: Json): TValid[String, TSchema] =
    jsonAST match {
      case Json.Obj(fields) => for {
          fields <- TValid.foreach(fields.toList) { case (name, value) => toTSchema(value).map(TSchema.Field(name, _)) }
        } yield TSchema.obj(fields)

      case Json.Arr(elements) => for {
          chunk  <- TValid.foreachChunk(elements)(toTSchema)
          schema <- unify(chunk.toList)
        } yield schema

      case Json.Bool(_) => TValid.succeed(TSchema.Boolean)
      case Json.Str(_)  => TValid.succeed(TSchema.String)
      case Json.Num(_)  => TValid.succeed(TSchema.Int)
      case Json.Null    => TValid.unsupported("NULL", "TSchema")
    }

  final def toTSchema(json: String): TValid[String, TSchema] =
    for {
      jsonAST <- TValid.fromEither(Json.decoder.decodeJson(json))
      tSchema <- toTSchema(jsonAST)
    } yield tSchema
}
