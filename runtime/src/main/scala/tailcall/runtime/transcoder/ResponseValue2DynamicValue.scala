package tailcall.runtime.transcoder

import zio.schema.DynamicValue

object ResponseValue2DynamicValue {
  import caliban.ResponseValue
  import caliban.ResponseValue.{StreamValue, ListValue => ResponseList, ObjectValue => ResponseObject}
  import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
  import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
  import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

  def fromResponseValue(input: ResponseValue): DynamicValue = {
    input match {
      case ResponseList(values)    => DynamicValue(values.map(fromResponseValue(_)))
      case ResponseObject(fields)  => DynamicValue(fields.toMap.map { case (k, v) => k -> fromResponseValue(v) })
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
      case StreamValue(_)          => DynamicValue.NoneValue
    }
  }
}
