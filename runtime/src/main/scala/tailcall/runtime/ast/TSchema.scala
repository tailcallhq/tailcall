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
  final def |(other: TSchema): TSchema = TSchema.Union(List(self, other))
  final def &(other: TSchema): TSchema = TSchema.Intersection(List(self, other))

  final def =:=(other: TSchema): Boolean = self <:< other && other <:< self
  final def <:<(other: TSchema): Boolean = TSchema.isSubType(self, other)
  final def arr: TSchema                 = TSchema.arr(self)

  final def isArray: Boolean =
    self match {
      case TSchema.Arr(_) => true
      case _              => false
    }

  final def isNullable: Boolean =
    self match {
      case _: TSchema.Optional => true
      case _                   => false
    }

  final def opt: TSchema = TSchema.opt(self)

  final def tag: String =
    self match {
      case TSchema.Obj(_)          => "Object"
      case TSchema.Arr(_)          => "Array"
      case TSchema.Optional(_)     => "Optional"
      case TSchema.Union(_)        => "Union"
      case TSchema.Intersection(_) => "Intersection"
      case TSchema.String          => "String"
      case TSchema.Int             => "Integer"
      case TSchema.Boolean         => "Boolean"
    }
}

object TSchema {

  def arr(item: TSchema): TSchema = TSchema.Arr(item)

  def bool: TSchema = TSchema.Boolean

  def empty: TSchema = TSchema.Obj(Nil)

  def int: TSchema = TSchema.Int

  def union(items: List[TSchema]): TSchema = Union(items)

  def intersection(items: List[TSchema]): TSchema = Intersection(items)

  def obj(fields: List[TSchema.Field]): TSchema = TSchema.Obj(fields.toList)

  def obj(fields: (String, TSchema)*): TSchema =
    TSchema.Obj(fields.map { case (name, schema) => TSchema.Field(name, schema) }.toList)

  def opt(schema: TSchema): TSchema =
    schema match {
      case Optional(_) => schema
      case _           => Optional(schema)
    }

  def string: TSchema = TSchema.String

  def toZIOSchema(schema: TSchema): Schema[_] =
    schema match {
      case TSchema.String      => Schema[String]
      case TSchema.Int         => Schema[Int]
      case TSchema.Boolean     => Schema[Boolean]
      case TSchema.Optional(s) => toZIOSchema(s).optional
      case Obj(fields)         =>
        val nFields = Chunk.from(fields).map(f => Labelled(f.name, toZIOSchema(f.schema).ast))
        ExtensibleMetaSchema.Product(TypeId.Structural, NodePath.empty, nFields).toSchema
      case Arr(item)           => Schema.chunk(toZIOSchema(item))
      case Union(_)            => ???
      case Intersection(_)     => ???
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
      case (TSchema.String, TSchema.String)   => true
      case (TSchema.Int, TSchema.Int)         => true
      case (TSchema.Boolean, TSchema.Boolean) => true
      case (Optional(t1), Optional(t2))       => isSubType(t1, t2)
      case (Obj(fields1), Obj(fields2))       => checkFields(fields1, fields2)
      case (Arr(item1), Arr(item2))           => isSubType(item1, item2)
      case (Union(schemas), _)                => schemas.forall(isSubType(_, s2))
      case (Intersection(schemas), _)         => schemas.exists(isSubType(_, s2))
      case (_, Union(schemas))                => schemas.exists(isSubType(s1, _))
      case (_, Intersection(schemas))         => schemas.forall(isSubType(s1, _))
      case _                                  => false
    }
  }

  @jsonHint("object")
  final case class Obj(fields: List[Field]) extends TSchema

  @jsonHint("array")
  final case class Arr(@jsonField("item") schema: TSchema) extends TSchema

  final case class Field(name: String, schema: TSchema)

  @jsonHint("optional")
  final case class Optional(schema: TSchema) extends TSchema

  @jsonHint("union")
  final case class Union(items: List[TSchema]) extends TSchema

  @jsonHint("intersection")
  final case class Intersection(items: List[TSchema]) extends TSchema

  @jsonHint("String")
  case object String extends TSchema

  @jsonHint("Integer")
  case object Int extends TSchema

  @jsonHint("Boolean")
  case object Boolean extends TSchema

  implicit lazy val fieldSchema: JsonCodec[TSchema.Field]    = DeriveJsonCodec.gen[TSchema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[TSchema] = zio.json.DeriveJsonCodec.gen[TSchema]
}
