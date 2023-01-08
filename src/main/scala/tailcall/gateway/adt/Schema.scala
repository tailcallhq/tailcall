package tailcall.gateway.adt

import zio.json._

@jsonDiscriminator("type")
sealed trait Schema
object Schema {

  sealed trait Scalar extends Schema

  @jsonHint("String")
  case object Str extends Scalar

  @jsonHint("Integer")
  case object Int extends Scalar

  @jsonHint("ID")
  case object Id extends Scalar

  @jsonHint("null")
  case object Null extends Scalar

  @jsonHint("object")
  final case class Obj(fields: List[Field]) extends Schema

  @jsonHint("array")
  final case class Arr(item: Schema) extends Schema

  final case class Field(name: String, schema: Schema, required: Boolean = false)

  @jsonHint("union")
  final case class Union(types: List[Schema]) extends Schema

  @jsonHint("intersect")
  final case class Intersection(types: List[Schema]) extends Schema

  implicit lazy val fieldSchema: JsonCodec[Schema.Field]    = DeriveJsonCodec.gen[Schema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[Schema] = zio.json.DeriveJsonCodec.gen[Schema]
}
