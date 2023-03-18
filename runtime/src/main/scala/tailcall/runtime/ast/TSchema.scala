package tailcall.runtime.ast

import zio.Chunk
import zio.json._
import zio.schema.meta.ExtensibleMetaSchema.Labelled
import zio.schema.meta.{ExtensibleMetaSchema, NodePath}
import zio.schema.{Schema, StandardType, TypeId}

/**
 * Represents the structure of a value. It allows us to
 * perform structural type checks.
 */
@jsonDiscriminator("type")
sealed trait TSchema {
  self =>
  def &(other: TSchema): TSchema = TSchema.Intersection(self, other)
  def |(other: TSchema): TSchema = TSchema.Union(self, other)

  def <:<(other: TSchema): Boolean = TSchema.isSubType(self, other)
  def =:=(other: TSchema): Boolean = self <:< other && other <:< self
  def arr: TSchema                 = TSchema.arr(self)
}

object TSchema {

  @jsonHint("String")
  case object String extends TSchema

  @jsonHint("Integer")
  case object Int extends TSchema

  @jsonHint("null")
  case object Null extends TSchema

  @jsonHint("Unit")
  case object Unit extends TSchema

  @jsonHint("Boolean")
  case object Boolean extends TSchema

  @jsonHint("object")
  final case class Obj(fields: List[Field]) extends TSchema

  @jsonHint("array")
  final case class Arr(item: TSchema) extends TSchema

  final case class Field(name: String, schema: TSchema)

  @jsonHint("union")
  final case class Union(self: TSchema, other: TSchema) extends TSchema

  @jsonHint("intersect")
  final case class Intersection(self: TSchema, other: TSchema) extends TSchema

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
      case (_, TSchema.Null)                  => true
      case (TSchema.Null, _)                  => false
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

  def string: TSchema = TSchema.String
  def int: TSchema    = TSchema.Int
  def `null`: TSchema = TSchema.Null
  def unit: TSchema   = TSchema.Unit

  def bool: TSchema = TSchema.Boolean

  def obj(fields: (String, TSchema)*): TSchema =
    TSchema.Obj(fields.map { case (name, schema) => TSchema.Field(name, schema) }.toList)

  def obj(fields: List[TSchema.Field]): TSchema = TSchema.Obj(fields.toList)

  def arr(item: TSchema): TSchema = TSchema.Arr(item)

  def fromZIOSchema(schema: Schema[_]): TSchema =
    schema.ast match {
      case ExtensibleMetaSchema.Product(_, _, fields, _) =>
        val nfields = fields.map(f => TSchema.Field(f.label, fromZIOSchema(f.schema.toSchema)))
        TSchema.Obj(nfields.toList)

      case ExtensibleMetaSchema.Tuple(_, _, _, _)      => ???
      case ExtensibleMetaSchema.Sum(_, _, _, _)        => ???
      case ExtensibleMetaSchema.Either(_, _, _, _)     => ???
      case ExtensibleMetaSchema.FailNode(_, _, _)      => ???
      case ExtensibleMetaSchema.ListNode(_, _, _)      => ???
      case ExtensibleMetaSchema.Dictionary(_, _, _, _) => ???
      case ExtensibleMetaSchema.Value(valueType, _, _) => valueType match {
          case StandardType.UnitType           => TSchema.string
          case StandardType.StringType         => TSchema.string
          case StandardType.BoolType           => TSchema.bool
          case StandardType.ByteType           => TSchema.string
          case StandardType.ShortType          => TSchema.string
          case StandardType.IntType            => TSchema.int
          case StandardType.LongType           => TSchema.string
          case StandardType.FloatType          => TSchema.string
          case StandardType.DoubleType         => TSchema.string
          case StandardType.BinaryType         => TSchema.string
          case StandardType.CharType           => TSchema.string
          case StandardType.UUIDType           => TSchema.string
          case StandardType.BigDecimalType     => TSchema.string
          case StandardType.BigIntegerType     => TSchema.string
          case StandardType.DayOfWeekType      => TSchema.string
          case StandardType.MonthType          => TSchema.string
          case StandardType.MonthDayType       => TSchema.string
          case StandardType.PeriodType         => TSchema.string
          case StandardType.YearType           => TSchema.string
          case StandardType.YearMonthType      => TSchema.string
          case StandardType.ZoneIdType         => TSchema.string
          case StandardType.ZoneOffsetType     => TSchema.string
          case StandardType.DurationType       => TSchema.string
          case StandardType.InstantType        => TSchema.string
          case StandardType.LocalDateType      => TSchema.string
          case StandardType.LocalTimeType      => TSchema.string
          case StandardType.LocalDateTimeType  => TSchema.string
          case StandardType.OffsetTimeType     => TSchema.string
          case StandardType.OffsetDateTimeType => TSchema.string
          case StandardType.ZonedDateTimeType  => TSchema.string
        }
      case ExtensibleMetaSchema.Ref(_, _, _)           => ???
      case ExtensibleMetaSchema.Known(_, _, _)         => ???
    }

  def toZIOSchema(schema: TSchema): Schema[_] =
    schema match {
      case TSchema.String  => Schema[String]
      case TSchema.Int     => Schema[Int]
      case TSchema.Null    => ???
      case TSchema.Unit    => Schema[Unit]
      case TSchema.Boolean => Schema[Boolean]

      case Obj(fields) =>
        val nfields = Chunk.from(fields).map(f => Labelled(f.name, toZIOSchema(f.schema).ast))
        ExtensibleMetaSchema.Product(TypeId.Structural, NodePath.empty, nfields).toSchema

      case Arr(item)          => Schema.chunk(toZIOSchema(item))
      case Union(_, _)        => ???
      case Intersection(_, _) => ???
    }

  implicit lazy val fieldSchema: JsonCodec[TSchema.Field]    = DeriveJsonCodec.gen[TSchema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[TSchema] = zio.json.DeriveJsonCodec.gen[TSchema]
}
