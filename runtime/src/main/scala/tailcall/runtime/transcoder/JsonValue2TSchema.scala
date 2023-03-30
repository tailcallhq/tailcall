package tailcall.runtime.transcoder

import tailcall.runtime.ast.TSchema
import tailcall.runtime.internal.TValid
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

  def unify(seq: TSchema*): TValid[String, TSchema] = unify(seq.toList)

  def unify(list: List[TSchema]): TValid[String, TSchema] = {
    list match {
      case Nil          => TValid.succeed(TSchema.empty) // Todo: Handle Errors in a better way
      case head :: Nil  => TValid.succeed(head)
      case head :: tail => unify(tail: _*).flatMap(unify2(head, _))
    }
  }

  final def toTSchema(jsonAST: Json): TValid[String, TSchema] = {
    jsonAST match {
      case Json.Obj(fields) => for {
          fields <- TValid.foreach(fields.toList) { case (name, value) => toTSchema(value).map(TSchema.Field(name, _)) }
        } yield TSchema.obj(fields)

      case Json.Arr(element) => for {
          chunk  <- TValid.foreachChunk(element)(json => toTSchema(json))
          schema <- unify(chunk.toList: _*)
        } yield schema.arr

      case Json.Bool(_) => TValid.succeed(TSchema.Boolean)
      case Json.Str(_)  => TValid.succeed(TSchema.String)
      case Json.Num(_)  => TValid.succeed(TSchema.Int)
      case Json.Null    => TValid.succeed(TSchema.obj())
    }
  }

  /**
   * Unifies two schemas into a single schema that is a
   * supertype of both. The unify function is different from
   * the union function because it is not just combining two
   * types into a single Union type. Instead, it is creating
   * a new schema that includes all the properties of both
   * input schemas. This is done to reduce unnecessary
   * unions. Incase of a conflict, the second schema is
   * selected.
   */
  private def unify2(a: TSchema, b: TSchema): TValid[String, TSchema] =
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
              case (Some(s1), Some(s2)) => unify2(s1, s2)
              case (Some(s1), None)     => TValid.succeed(s1.opt)
              case (None, Some(s2))     => TValid.succeed(s2.opt)
              case (None, None)         => TValid.fail(s"Key ${key} should be present in one of the maps")
            }

            fieldDesc.map(TSchema.Field(key, _))
          }
        } yield TSchema.obj(fields.toList)

      case (TSchema.Arr(item1), TSchema.Arr(item2)) => unify2(item1, item2).map(TSchema.arr(_))
      case (a, TSchema.Obj(Nil))                    => TValid.succeed(a.opt)
      case (TSchema.Obj(Nil), b)                    => TValid.succeed(b.opt)
      case (TSchema.Optional(a), b)                 => unify2(a, b).map(_.opt)
      case (a, TSchema.Optional(b))                 => unify2(a, b).map(_.opt)
      case (_, b)                                   => TValid.succeed(b)
    }
}
object JsonValue2TSchema {
  sealed trait Error
  object Error {}
}
