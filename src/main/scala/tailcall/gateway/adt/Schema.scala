package tailcall.gateway.adt

import zio.json._

@jsonDiscriminator("type")
sealed trait Schema {
  self =>
  def &(other: Schema): Schema = Schema.Intersection(self, other)
  def |(other: Schema): Schema = Schema.Union(self, other)

  def <:<(other: Schema): Boolean = Schema.isSubType(self, other)
  def =:=(other: Schema): Boolean = self <:< other && other <:< self
}

object Schema {
  sealed trait Scalar extends Schema

  object Scalar {
    @jsonHint("String")
    case object Str extends Scalar

    @jsonHint("Integer")
    case object Int extends Scalar

    @jsonHint("ID")
    case object Id extends Scalar

    @jsonHint("null")
    case object Null extends Scalar
  }

  @jsonHint("object")
  final case class Obj(fields: List[Field]) extends Schema

  @jsonHint("array")
  final case class Arr(item: Schema) extends Schema

  final case class Field(name: String, schema: Schema, required: Boolean = false)

  @jsonHint("union")
  final case class Union(self: Schema, other: Schema) extends Schema

  @jsonHint("intersect")
  final case class Intersection(self: Schema, other: Schema) extends Schema

  // TODO: add unit tests
  private def isSubType(s1: Schema, s2: Schema): Boolean = {
    def checkFields(fields1: List[Field], fields2: List[Field]): Boolean = {
      fields2.forall { f2 =>
        fields1.exists { f1 =>
          f1.name == f2.name &&
          isSubType(f1.schema, f2.schema) &&
          (!f2.required || f1.required)
        }
      }
    }

    (s1, s2) match {
      case (_, Scalar.Null) =>
        true

      case (Scalar.Null, _) =>
        false

      case (s1: Scalar, s2: Scalar) =>
        s1 == s2

      case (Obj(fields1), Obj(fields2)) =>
        checkFields(fields1, fields2)

      case (Arr(item1), Arr(item2)) =>
        isSubType(item1, item2)

      case (Union(s1a, s1b), _) =>
        isSubType(s1a, s2) || isSubType(s1b, s2)

      case (Intersection(s1a, s1b), _) =>
        isSubType(s1a, s2) && isSubType(s1b, s2)

      case _ =>
        false
    }
  }

  def string: Schema = Schema.Scalar.Str
  def int: Schema    = Schema.Scalar.Int
  def `null`: Schema = Schema.Scalar.Null

  implicit lazy val fieldSchema: JsonCodec[Schema.Field]    = DeriveJsonCodec.gen[Schema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[Schema] = zio.json.DeriveJsonCodec.gen[Schema]
}
