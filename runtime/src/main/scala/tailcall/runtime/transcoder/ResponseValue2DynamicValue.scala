package tailcall.runtime.transcoder

import tailcall.runtime.internal.DynamicValueUtil
import zio.Chunk
import zio.schema.DynamicValue

trait ResponseValue2DynamicValue {
  import caliban.ResponseValue
  import caliban.ResponseValue.{StreamValue, ListValue => ResponseList, ObjectValue => ResponseObject}
  import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
  import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
  import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

  private def run(input: ResponseValue): TValid[String, DynamicValue] = {
    input match {
      case ResponseList(values)    => TValid.foreachChunk(Chunk.from(values))(run).map(DynamicValue.Sequence)
      case ResponseObject(fields)  => TValid.foreach(fields) { case (k, v) => run(v).map(k -> _) }
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
      case EnumValue(_)            => TValid.fail("Can not transcode EnumValue to DynamicValue")
      case StreamValue(_)          => TValid.fail("Can not transcode StreamValue to DynamicValue")
    }
  }

  def toDynamicValue(input: ResponseValue): TValid[String, DynamicValue] = run(input)
}
