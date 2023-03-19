package tailcall.runtime.transcoder

import caliban.ResponseValue
import tailcall.runtime.internal.DynamicValueUtil
import zio.Chunk
import zio.schema.DynamicValue

object ResponseValue2DynamicValue extends Transcoder[ResponseValue, String, DynamicValue] {
  import caliban.ResponseValue
  import caliban.ResponseValue.{StreamValue, ListValue => ResponseList, ObjectValue => ResponseObject}
  import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
  import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
  import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

  def run(input: ResponseValue): TExit[String, DynamicValue] = {
    input match {
      case ResponseList(values)    => TExit.foreachChunk(Chunk.from(values))(run).map(DynamicValue.Sequence)
      case ResponseObject(fields)  => TExit.foreach(fields) { case (k, v) => run(v).map(k -> _) }
          .map(entries => DynamicValueUtil.record(entries: _*))
      case StringValue(value)      => TExit.succeed(DynamicValue(value))
      case NullValue               => TExit.succeed(DynamicValue(()))
      case BooleanValue(value)     => TExit.succeed(DynamicValue(value))
      case BigDecimalNumber(value) => TExit.succeed(DynamicValue(value))
      case DoubleNumber(value)     => TExit.succeed(DynamicValue(value))
      case FloatNumber(value)      => TExit.succeed(DynamicValue(value))
      case BigIntNumber(value)     => TExit.succeed(DynamicValue(value))
      case IntNumber(value)        => TExit.succeed(DynamicValue(value))
      case LongNumber(value)       => TExit.succeed(DynamicValue(value))
      case EnumValue(_)            => TExit.fail("Can not transcode EnumValue to DynamicValue")
      case StreamValue(_)          => TExit.fail("Can not transcode StreamValue to DynamicValue")
    }
  }
}
