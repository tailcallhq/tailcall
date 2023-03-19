package tailcall.runtime.transcoder

import caliban.InputValue
import tailcall.runtime.internal.DynamicValueUtil
import zio.Chunk
import zio.schema.DynamicValue

object InputValue2DynamicValue extends Transcoder[InputValue, String, DynamicValue] {

  import caliban.InputValue.{ListValue, ObjectValue, VariableValue}
  import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
  import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
  import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

  override def run(input: InputValue): TValid[String, DynamicValue] = {
    input match {
      case ListValue(values)       => TValid.foreachChunk(Chunk.from(values))(run).map(DynamicValue.Sequence)
      case ObjectValue(fields)     => TValid.foreachIterable(fields) { case (k, v) => run(v).map(k -> _) }
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
      case EnumValue(_)            => TValid.fail("Can not transcode EnumValue to DynamicValue")
      case VariableValue(_)        => TValid.fail("Can not transcode VariableValue to DynamicValue")
    }
  }
}
