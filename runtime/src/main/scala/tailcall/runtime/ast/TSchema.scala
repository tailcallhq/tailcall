package tailcall.runtime.ast

import zio.Chunk
import zio.json._
import zio.schema.meta.ExtensibleMetaSchema.Labelled
import zio.schema.meta.{ExtensibleMetaSchema, NodePath}
import zio.schema.{Schema, TypeId}

/**
 * Represents the structure of a value. It allows us to
 * perform structural type checks.
 */
@jsonDiscriminator("type")
sealed trait TSchema {
  self =>
  final def &(other: TSchema): TSchema   = TSchema.Intersection(self, other)
  final def |(other: TSchema): TSchema   = TSchema.Union(self, other)
  final def =:=(other: TSchema): Boolean = self <:< other && other <:< self
  final def <:<(other: TSchema): Boolean = TSchema.isSubType(self, other)
  final def arr: TSchema                 = TSchema.arr(self)
  final def isNullable: Boolean          = TSchema.NULL <:< self
  final def isArray: Boolean             =
    self match {
      case TSchema.Arr(_) => true
      case _              => false
    }

  /**
   * Unifies two schemas into a single schema that is a
   * supertype of both. The unify function is different from
   * the union function because it is not just combining two
   * types into a single Union type. Instead, it is creating
   * a new schema that includes all the properties of both
   * input schemas. This is done to reduce unnecessary
   * unions.
   */
  final def unify(other: TSchema): TSchema = TSchema.unify(self, other)
}

object TSchema {

  def string: TSchema = TSchema.String

  def int: TSchema = TSchema.Int

  def unit: TSchema = TSchema.Unit

  def bool: TSchema = TSchema.Boolean

  def obj(fields: (String, TSchema)*): TSchema =
    TSchema.Obj(fields.map { case (name, schema) => TSchema.Field(name, schema) }.toList)

  def obj(fields: List[TSchema.Field]): TSchema = TSchema.Obj(fields.toList)

  def arr(item: TSchema): TSchema = TSchema.Arr(item)

  def toZIOSchema(schema: TSchema): Schema[_] =
    schema match {
      case TSchema.String  => Schema[String]
      case TSchema.Int     => Schema[Int]
      case TSchema.NULL    => ???
      case TSchema.Unit    => Schema[Unit]
      case TSchema.Boolean => Schema[Boolean]

      case Obj(fields) =>
        val nfields = Chunk.from(fields).map(f => Labelled(f.name, toZIOSchema(f.schema).ast))
        ExtensibleMetaSchema.Product(TypeId.Structural, NodePath.empty, nfields).toSchema

      case Arr(item)          => Schema.chunk(toZIOSchema(item))
      case Union(_, _)        => ???
      case Intersection(_, _) => ???
    }

  private def unify(a: TSchema, b: TSchema): TSchema =
    (a, b) match {
      case (TSchema.Int, TSchema.Int)                   => TSchema.Int
      case (TSchema.String, TSchema.String)             => TSchema.String
      case (TSchema.Boolean, TSchema.Boolean)           => TSchema.Boolean
      case (TSchema.Unit, TSchema.Unit)                 => TSchema.Unit
      case (TSchema.NULL, TSchema.NULL)                 => TSchema.NULL
      case (TSchema.Obj(fields1), TSchema.Obj(fields2)) =>
        val field1Map: Map[String, TSchema] = fields1.map(f => f.name -> f.schema).toMap
        val field2Map: Map[String, TSchema] = fields2.map(f => f.name -> f.schema).toMap
        TSchema.Obj((field1Map.keys ++ field2Map.keys).toList.map { key =>
          TSchema.Field(
            key,
            (field1Map.get(key), field2Map.get(key)) match {
              case (Some(s1), Some(s2)) => unify(s1, s2)
              case (Some(s1), None)     => s1 | TSchema.NULL
              case (None, Some(s2))     => s2 | TSchema.NULL
              case (None, None) => throw new IllegalStateException(s"Key ${key} should be present in one of the maps")
            },
          )
        })

      case (TSchema.Arr(item1), TSchema.Arr(item2)) => TSchema.Arr(unify(item1, item2))
      case (TSchema.Union(a, b), c)                 => a unify b unify c
      case (a, TSchema.Union(b, c))                 => a unify b unify c
      case (a, b)                                   => a | b
    }

  // TODO: add unit tests
  private def isSubType(s1: TSchema, s2: TSchema): Boolean = {
    def checkFields(fields1: List[Field], fields2: List[Field]): Boolean = {
      fields2.forall { f2 =>
        fields1.exists { f1 =>
          f1.name == f2.name &&
          isSubType(f1.schema, f2.schema)
        }
      }
    }

    (s1, s2) match {
      case (_, TSchema.NULL)                  => true
      case (TSchema.NULL, _)                  => false
      case (TSchema.String, TSchema.String)   => true
      case (TSchema.Int, TSchema.Int)         => true
      case (TSchema.Unit, TSchema.Unit)       => true
      case (TSchema.Boolean, TSchema.Boolean) => true
      case (Obj(fields1), Obj(fields2))       => checkFields(fields1, fields2)
      case (Arr(item1), Arr(item2))           => isSubType(item1, item2)
      case (Union(s1a, s1b), _)               => isSubType(s1a, s2) || isSubType(s1b, s2)
      case (Intersection(s1a, s1b), _)        => isSubType(s1a, s2) && isSubType(s1b, s2)
      case _                                  => false
    }
  }

  @jsonHint("object")
  final case class Obj(fields: List[Field]) extends TSchema

  @jsonHint("array")
  final case class Arr(item: TSchema) extends TSchema

  final case class Field(name: String, schema: TSchema)

  @jsonHint("union")
  final case class Union(self: TSchema, other: TSchema) extends TSchema

  @jsonHint("intersect")
  final case class Intersection(self: TSchema, other: TSchema) extends TSchema

  @jsonHint("String")
  case object String extends TSchema

  @jsonHint("Integer")
  case object Int extends TSchema

  @jsonHint("null")
  case object NULL extends TSchema

  @jsonHint("Unit")
  case object Unit extends TSchema

  @jsonHint("Boolean")
  case object Boolean extends TSchema

  implicit lazy val fieldSchema: JsonCodec[TSchema.Field]    = DeriveJsonCodec.gen[TSchema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[TSchema] = zio.json.DeriveJsonCodec.gen[TSchema]
}
