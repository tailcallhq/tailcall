package tailcall.runtime.transcoder

import tailcall.runtime.internal.TValid
import zio.json.ast.Json
import zio.schema.{DynamicValue, TypeId}

import scala.collection.immutable.ListMap

trait Json2DynamicValue {
  final def toDynamicValue(json: Json): TValid[String, DynamicValue] = {
    json match {
      case Json.Obj(fields)   => TValid.foreach(fields.toList) { case (k, v) => toDynamicValue(v).map(k -> _) }
          .map(ListMap.from(_)).map(DynamicValue.Record(TypeId.Structural, _))
      case Json.Arr(elements) => TValid.foreachChunk(elements)(toDynamicValue).map(DynamicValue.Sequence)
      case Json.Bool(value)   => TValid.succeed(DynamicValue(value))
      case Json.Str(value)    => TValid.succeed(DynamicValue(value))
      case Json.Num(value)    => TValid.succeed(DynamicValue(value))
      case Json.Null          => TValid.succeed(DynamicValue(()))
    }
  }

}
