package tailcall.gateway.ast

import zio.json._

/**
 * Represents the structure of a value. It allows us to
 * perform structural type checks.
 */
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

    @jsonHint("ID")
    case object Id extends Scalar

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

  def str: TSchema    = TSchema.Scalar.Str
  def int: TSchema    = TSchema.Scalar.Int
  def `null`: TSchema = TSchema.Scalar.Null
  def unit: TSchema   = TSchema.Scalar.Unit

  def bool: TSchema = TSchema.Scalar.Boolean

  def obj(fields: (String, TSchema)*): TSchema =
    TSchema.Obj(fields.map { case (name, schema) => TSchema.Field(name, schema) }.toList)

  def obj(fields: List[TSchema.Field]): TSchema = TSchema.Obj(fields.toList)

  def arr(item: TSchema): TSchema = TSchema.Arr(item)

  implicit lazy val idSchema: JsonCodec[TSchema.Id]          = DeriveJsonCodec.gen[TSchema.Id]
  implicit lazy val fieldSchema: JsonCodec[TSchema.Field]    = DeriveJsonCodec.gen[TSchema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[TSchema] = zio.json.DeriveJsonCodec.gen[TSchema]
}
