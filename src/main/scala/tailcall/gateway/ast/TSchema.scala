package tailcall.gateway.ast

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
  sealed trait Scalar extends TSchema

  object Scalar {
    @jsonHint("String")
    case object Str extends Scalar

    @jsonHint("Integer")
    case object Int extends Scalar

    @jsonHint("null")
    case object Null extends Scalar

    @jsonHint("Unit")
    case object Unit extends Scalar

    @jsonHint("Boolean")
    case object Boolean extends Scalar
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
      case (_, Scalar.Null) => true

      case (Scalar.Null, _) => false

      case (s1: Scalar, s2: Scalar) => s1 == s2

      case (Obj(fields1), Obj(fields2)) => checkFields(fields1, fields2)

      case (Arr(item1), Arr(item2)) => isSubType(item1, item2)

      case (Union(s1a, s1b), _) => isSubType(s1a, s2) || isSubType(s1b, s2)

      case (Intersection(s1a, s1b), _) => isSubType(s1a, s2) && isSubType(s1b, s2)

      case _ => false
    }
  }

  sealed trait Id
  object Id {
    case class Named(name: String) extends Id
    case object Structural         extends Id
  }

  def string: TSchema = TSchema.Scalar.Str
  def int: TSchema    = TSchema.Scalar.Int
  def `null`: TSchema = TSchema.Scalar.Null
  def unit: TSchema   = TSchema.Scalar.Unit

  def bool: TSchema = TSchema.Scalar.Boolean

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
      case scalar: Scalar => scalar match {
          case Scalar.Str     => Schema[String]
          case Scalar.Int     => Schema[Int]
          case Scalar.Null    => ???
          case Scalar.Unit    => Schema[Unit]
          case Scalar.Boolean => Schema[Boolean]
        }
      case Obj(fields)    =>
        val nfields = Chunk.from(fields).map(f => Labelled(f.name, toZIOSchema(f.schema).ast))
        ExtensibleMetaSchema.Product(TypeId.Structural, NodePath.empty, nfields).toSchema

      case Arr(item)          => Schema.chunk(toZIOSchema(item))
      case Union(_, _)        => ???
      case Intersection(_, _) => ???
    }

  implicit lazy val idSchema: JsonCodec[TSchema.Id]          = DeriveJsonCodec.gen[TSchema.Id]
  implicit lazy val fieldSchema: JsonCodec[TSchema.Field]    = DeriveJsonCodec.gen[TSchema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[TSchema] = zio.json.DeriveJsonCodec.gen[TSchema]
}
