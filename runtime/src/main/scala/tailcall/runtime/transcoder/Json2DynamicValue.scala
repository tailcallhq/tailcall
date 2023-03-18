package tailcall.runtime.transcoder

import zio.json.ast.Json
import zio.schema.{DynamicValue, TypeId}

import scala.collection.immutable.ListMap

object Json2DynamicValue {
  def fromJson(json: Json): DynamicValue =
    json match {
      case Json.Obj(fields)   => DynamicValue
          .Record(TypeId.Structural, ListMap.from(fields.map { case (k, v) => k -> fromJson(v) }))
      case Json.Arr(elements) => DynamicValue(elements.map(fromJson))
      case Json.Bool(value)   => DynamicValue(value)
      case Json.Str(value)    => DynamicValue(value)
      case Json.Num(value)    => DynamicValue(value)
      case Json.Null          => DynamicValue.NoneValue
    }
}
