package tailcall.runtime.transcoder

import caliban.InputValue
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.transcoder.Transcoder.TExit
import zio.Chunk
import zio.schema.DynamicValue

object InputValue2DynamicValue {

  import caliban.InputValue.{ListValue, ObjectValue, VariableValue}
  import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
  import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
  import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

  def fromInputValue(input: InputValue): TExit[String, DynamicValue] = {
    input match {
      case ListValue(values)       => TExit.foreachChunk(Chunk.from(values))(fromInputValue).map(DynamicValue.Sequence)
      case ObjectValue(fields)     => TExit.foreachIterable(fields) { case (k, v) => fromInputValue(v).map(k -> _) }
          .map(entries => DynamicValueUtil.record(entries.toList: _*))
      case StringValue(value)      => TExit.succeed(DynamicValue(value))
      case NullValue               => TExit.fail("Can not transcode NullValue to DynamicValue")
      case BooleanValue(value)     => TExit.succeed(DynamicValue(value))
      case BigDecimalNumber(value) => TExit.succeed(DynamicValue(value))
      case DoubleNumber(value)     => TExit.succeed(DynamicValue(value))
      case FloatNumber(value)      => TExit.succeed(DynamicValue(value))
      case BigIntNumber(value)     => TExit.succeed(DynamicValue(value))
      case IntNumber(value)        => TExit.succeed(DynamicValue(value))
      case LongNumber(value)       => TExit.succeed(DynamicValue(value))
      case EnumValue(_)            => TExit.fail("Can not transcode EnumValue to DynamicValue")
      case VariableValue(_)        => TExit.fail("Can not transcode VariableValue to DynamicValue")
    }
  }
}
