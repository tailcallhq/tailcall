package tailcall.runtime.transcoder

import caliban.InputValue
import zio.schema.DynamicValue

object InputValue2DynamicValue {

  import caliban.InputValue.{VariableValue, ListValue => InputList, ObjectValue => InputObject}
  import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
  import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
  import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

  def fromInputValue(input: InputValue): DynamicValue = {
    input match {
      case InputList(values)       => DynamicValue(values.map(fromInputValue(_)))
      case InputObject(fields)     => DynamicValue(fields.map { case (k, v) => k -> fromInputValue(v) })
      case StringValue(value)      => DynamicValue(value)
      case NullValue               => DynamicValue.NoneValue
      case BooleanValue(value)     => DynamicValue(value)
      case BigDecimalNumber(value) => DynamicValue(value)
      case DoubleNumber(value)     => DynamicValue(value)
      case FloatNumber(value)      => DynamicValue(value)
      case BigIntNumber(value)     => DynamicValue(value)
      case IntNumber(value)        => DynamicValue(value)
      case LongNumber(value)       => DynamicValue(value)
      case EnumValue(_)            => DynamicValue.NoneValue
      case VariableValue(_)        => DynamicValue.NoneValue
    }
  }
}
