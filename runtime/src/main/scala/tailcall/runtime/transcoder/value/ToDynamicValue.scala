package tailcall.runtime.transcoder.value

import caliban.ResponseValue
import caliban.ResponseValue.{ListValue => ResponseList, ObjectValue => ResponseObject, StreamValue}
import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}
import tailcall.runtime.internal.{DynamicValueUtil, TValid}
import zio.Chunk
import zio.json.ast.Json
import zio.schema.{DynamicValue, TypeId}

import scala.collection.immutable.ListMap
trait ToDynamicValue {

  final def toDynamicValue(input: ResponseValue): TValid[String, DynamicValue] = {
    input match {
      case ResponseList(values)    => TValid.foreachChunk(Chunk.from(values))(toDynamicValue).map(DynamicValue.Sequence)
      case ResponseObject(fields)  => TValid.foreach(fields) { case (k, v) => toDynamicValue(v).map(k -> _) }
          .map(entries => DynamicValueUtil.record(entries: _*))
      case StringValue(value)      => TValid.succeed(DynamicValue(value))
      case NullValue               => TValid.succeed(DynamicValue(()))
      case BooleanValue(value)     => TValid.succeed(DynamicValue(value))
      case BigDecimalNumber(value) => TValid.succeed(DynamicValue(value))
      case DoubleNumber(value)     => TValid.succeed(DynamicValue(value))
      case FloatNumber(value)      => TValid.succeed(DynamicValue(value))
      case BigIntNumber(value)     => TValid.succeed(DynamicValue(value))
      case IntNumber(value)        => TValid.succeed(DynamicValue(value))
      case LongNumber(value)       => TValid.succeed(DynamicValue(value))
      case EnumValue(value)        => TValid.succeed(DynamicValue(value))
      case StreamValue(_)          => TValid.fail("Can not transcode StreamValue to DynamicValue")
    }
  }

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

  import caliban.InputValue
  import caliban.InputValue.{ListValue, ObjectValue, VariableValue}
  import tailcall.runtime.internal.{DynamicValueUtil, TValid}
  import zio.Chunk
  import zio.schema.DynamicValue

  final def toDynamicValue(input: InputValue): TValid[String, DynamicValue] = {
    input match {
      case ListValue(values)       => TValid.foreachChunk(Chunk.from(values))(toDynamicValue).map(DynamicValue.Sequence)
      case ObjectValue(fields)     => TValid.foreachIterable(fields) { case (k, v) => toDynamicValue(v).map(k -> _) }
          .map(entries => DynamicValueUtil.record(entries.toList: _*))
      case StringValue(value)      => TValid.succeed(DynamicValue(value))
      case NullValue               => TValid.succeed(DynamicValue(()))
      case BooleanValue(value)     => TValid.succeed(DynamicValue(value))
      case BigDecimalNumber(value) => TValid.succeed(DynamicValue(value))
      case DoubleNumber(value)     => TValid.succeed(DynamicValue(value))
      case FloatNumber(value)      => TValid.succeed(DynamicValue(value))
      case BigIntNumber(value)     => TValid.succeed(DynamicValue(value))
      case IntNumber(value)        => TValid.succeed(DynamicValue(value))
      case LongNumber(value)       => TValid.succeed(DynamicValue(value))
      case EnumValue(value)        => TValid.succeed(DynamicValue(value))
      case VariableValue(_)        => TValid.fail("Can not transcode VariableValue to DynamicValue")
    }
  }
}
