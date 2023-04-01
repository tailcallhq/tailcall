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

  final def toTSchema(jsonAST: Json): TValid[String, TSchema] = {
    jsonAST match {
      case Json.Obj(fields) => for {
          valueSchema <- TValid.foreachChunk(fields)(field => toTSchema(field._2)).map(_.distinct)
          keys        <- TValid.succeed(fields.map(_._1).distinct)
          schema      <- valueSchema.headOption match {
            case Some(schema) if fields.length != 1 && valueSchema.length == 1 && keys.length == fields.length =>
              TValid.succeed(TSchema.dict(schema))
            case _ => TValid.foreachChunk(fields) { case (name, value) => toTSchema(value).map((name, _)) }
                .map(fields => TSchema.obj(fields.toMap))
          }
        } yield schema

      case Json.Arr(element) => for {
          chunk  <- TValid.foreachChunk(element)(json => toTSchema(json))
          schema <- unify(chunk.toList: _*).map(_.getOrElse(TSchema.String))
        } yield schema.arr

      case Json.Bool(_) => TValid.succeed(TSchema.Boolean)
      case Json.Str(_)  => TValid.succeed(TSchema.String)
      case Json.Num(_)  => TValid.succeed(TSchema.Int)
      case Json.Null    => TValid.succeed(TSchema.obj())
    }
  }

  final def unify(list: List[TSchema]): TValid[String, Option[TSchema]] = {
    var cache = Map.empty[(TSchema, TSchema), Option[TSchema]]

    def loop(list: List[TSchema]): TValid[String, Option[TSchema]] = {
      list match {
        case Nil          => TValid.none
        case head :: Nil  => TValid.some(head)
        case head :: tail => unify(tail: _*).flatMap {
            case Some(schema) => cache.get(head, schema) match {
                case Some(value) => TValid.succeed(value)
                case None        => for {
                    value <- unify2(head, schema)
                  } yield {
                    cache = cache.updated((head, schema), value)
                    value
                  }
              }

            case None => loop(tail)
          }
      }
    }
    loop(list)
  }

  final def unify(seq: TSchema*): TValid[String, Option[TSchema]] = unify(seq.toList)

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
  private def unify2(a: TSchema, b: TSchema): TValid[String, Option[TSchema]] = {
    (a, b) match {
      case (TSchema.Int, TSchema.Int)                   => TValid.some(TSchema.Int)
      case (TSchema.String, TSchema.String)             => TValid.some(TSchema.String)
      case (TSchema.Boolean, TSchema.Boolean)           => TValid.some(TSchema.Boolean)
      case (TSchema.Obj(fields1), TSchema.Obj(fields2)) =>
        val field1Map: Map[String, TSchema] = fields1.map(f => f._1 -> f._2).toMap
        val field2Map: Map[String, TSchema] = fields2.map(f => f._1 -> f._2).toMap

        for {
          fields <- TValid.foreachIterable(field1Map.keys ++ field2Map.keys) { key =>
            val schema = (field1Map.get(key), field2Map.get(key)) match {
              case (Some(s1), Some(s2)) => unify2(s1, s2).map(_.getOrElse(s2))
              case (Some(s1), None)     => TValid.succeed(s1.opt)
              case (None, Some(s2))     => TValid.succeed(s2.opt)
              case (None, None)         => TValid.fail(s"Key ${key} should be present in one of the maps")
            }

            schema.map((key, _))
          }
        } yield Option(TSchema.obj(fields.toMap))

      case (TSchema.Arr(item1), TSchema.Arr(item2)) => unify2(item1, item2).map(_.map(TSchema.arr))
      case (a, TSchema.Obj(map)) if map.isEmpty     => TValid.some(a.opt)
      case (TSchema.Obj(map), b) if map.isEmpty     => TValid.some(b.opt)
      case (TSchema.Optional(a), b)                 => unify2(a, b).map(_.map(_.opt))
      case (a, TSchema.Optional(b))                 => unify2(a, b).map(_.map(_.opt))
      case (_, _)                                   => TValid.none
    }
  }
}
