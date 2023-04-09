package tailcall.runtime.model

import zio.Chunk
import zio.json._
import zio.schema.meta.ExtensibleMetaSchema.Labelled
import zio.schema.meta.{ExtensibleMetaSchema, NodePath}
import zio.schema.{DeriveSchema, Schema, TypeId}

/**
 * Represents the structure of a value. It allows us to
 * perform structural type checks.
 */
@jsonDiscriminator("type")
sealed trait TSchema {
  self =>
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
      case _: TSchema.Opt => true
      case _              => false
    }

  final def opt: TSchema = TSchema.opt(self)

  final def tag: String =
    self match {
      case TSchema.Obj(_) => "Object"
      case TSchema.Arr(_) => "Array"
      case TSchema.Opt(_) => "Optional"
      case TSchema.Str    => "String"
      case TSchema.Num    => "Long"
      case TSchema.Bool   => "Boolean"
    }
}

object TSchema {

  def arr(item: TSchema): TSchema = TSchema.Arr(item)

  def bool: TSchema = TSchema.Bool

  def empty: TSchema = TSchema.Obj(Map.empty)

  def int: TSchema = TSchema.Num

  def obj(map: Map[String, TSchema]): TSchema = TSchema.Obj(map)

  def obj(fields: (String, TSchema)*): TSchema = TSchema.Obj(fields.toMap)

  def opt(schema: TSchema): TSchema =
    schema match {
      case Opt(_) => schema
      case _      => Opt(schema)
    }

  def string: TSchema = TSchema.Str

  def toZIOSchema(schema: TSchema): Schema[_] =
    schema match {
      case TSchema.Str    => Schema[String]
      case TSchema.Num    => Schema[Long]
      case TSchema.Bool   => Schema[Boolean]
      case TSchema.Opt(s) => toZIOSchema(s).optional
      case Obj(fields)    =>
        val nFields = Chunk.from(fields).map(f => Labelled(f._1, toZIOSchema(f._2).ast))
        ExtensibleMetaSchema.Product(TypeId.Structural, NodePath.empty, nFields).toSchema
      case Arr(item)      => Schema.chunk(toZIOSchema(item))
    }

  // TODO: add unit tests
  private def isSubType(s1: TSchema, s2: TSchema): Boolean = {
    def checkFields(fields1: Map[String, TSchema], fields2: Map[String, TSchema]): Boolean = {
      fields2.forall { f2 =>
        fields1.exists { f1 =>
          f1._1 == f2._1 &&
          isSubType(f1._2, f2._2)
        }
      }
    }

    (s1, s2) match {
      case (TSchema.Str, TSchema.Str)   => true
      case (TSchema.Num, TSchema.Num)   => true
      case (TSchema.Bool, TSchema.Bool) => true
      case (Obj(fields1), Obj(fields2)) => checkFields(fields1, fields2)
      case (Arr(item1), Arr(item2))     => isSubType(item1, item2)
      case _                            => false
    }
  }

  @jsonHint("object")
  final case class Obj(fields: Map[String, TSchema]) extends TSchema

  @jsonHint("array")
  final case class Arr(@jsonField("item") schema: TSchema) extends TSchema

  @jsonHint("optional")
  final case class Opt(schema: TSchema) extends TSchema

  @jsonHint("String")
  case object Str extends TSchema

  @jsonHint("Integer")
  case object Num extends TSchema

  @jsonHint("Boolean")
  case object Bool extends TSchema

  implicit lazy val schemaCodec: zio.json.JsonCodec[TSchema] = zio.json.DeriveJsonCodec.gen[TSchema]
  implicit lazy val schema: Schema[TSchema]                  = DeriveSchema.gen[TSchema]
}
