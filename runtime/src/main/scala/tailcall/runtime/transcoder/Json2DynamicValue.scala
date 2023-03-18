package tailcall.runtime.transcoder

import tailcall.runtime.transcoder.Transcoder.TExit
import zio.json.ast.Json
import zio.schema.{DynamicValue, TypeId}

import scala.collection.immutable.ListMap

object Json2DynamicValue {
  def fromJson(json: Json): TExit[String, DynamicValue] = {
    json match {
      case Json.Obj(fields)   => TExit.foreach(fields.toList) { case (k, v) => fromJson(v).map(k -> _) }
          .map(ListMap.from(_)).map(DynamicValue.Record(TypeId.Structural, _))
      case Json.Arr(elements) => TExit.foreachChunk(elements)(fromJson).map(DynamicValue.Sequence)
      case Json.Bool(value)   => TExit.succeed(DynamicValue(value))
      case Json.Str(value)    => TExit.succeed(DynamicValue(value))
      case Json.Num(value)    => TExit.succeed(DynamicValue(value))
      case Json.Null          => TExit.succeed(DynamicValue(()))
    }
  }
}
