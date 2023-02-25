package tailcall.gateway.internal

import caliban.{ResponseValue, Value}
import zio.schema.{DynamicValue, StandardType}

object DynamicValueUtil {
  def toValue(value: Any, standardType: StandardType[_]): ResponseValue =
    standardType match {
      case StandardType.StringType         => Value.StringValue(value.toString)
      case StandardType.IntType            => Value.IntValue(value.toString.toInt)
      case StandardType.MonthDayType       => Value.StringValue(value.toString)
      case StandardType.LocalDateTimeType  => Value.StringValue(value.toString)
      case StandardType.BoolType           => Value.BooleanValue(value.toString.toBoolean)
      case StandardType.LocalTimeType      => Value.StringValue(value.toString)
      case StandardType.OffsetDateTimeType => Value.StringValue(value.toString)
      case StandardType.MonthType          => Value.StringValue(value.toString)
      case StandardType.ShortType          => Value.IntValue(value.toString.toShort)
      case StandardType.ZoneIdType         => Value.StringValue(value.toString)
      case StandardType.BigDecimalType     => Value.StringValue(value.toString)
      case StandardType.YearType           => Value.IntValue(value.toString.toInt)
      case StandardType.ByteType           => Value.IntValue(value.toString.toByte)
      case StandardType.UUIDType           => Value.StringValue(value.toString)
      case StandardType.PeriodType         => Value.StringValue(value.toString)
      case StandardType.LongType           => Value.StringValue(value.toString)
      case StandardType.ZoneOffsetType     => Value.StringValue(value.toString)
      case StandardType.BigIntegerType     => Value.StringValue(value.toString)
      case StandardType.OffsetTimeType     => Value.StringValue(value.toString)
      case StandardType.UnitType           => Value.NullValue
      case StandardType.DoubleType         => Value.FloatValue(value.toString.toDouble)
      case StandardType.InstantType        => Value.StringValue(value.toString)
      case StandardType.FloatType          => Value.FloatValue(value.toString.toFloat)
      case StandardType.LocalDateType      => Value.StringValue(value.toString)
      case StandardType.ZonedDateTimeType  => Value.StringValue(value.toString)
      case StandardType.YearMonthType      => Value.StringValue(value.toString)
      case StandardType.CharType           => Value.StringValue(value.toString)
      case StandardType.BinaryType         => Value
          .StringValue(java.util.Base64.getEncoder.encodeToString(value.asInstanceOf[Array[Byte]]))
      case StandardType.DurationType       => Value.StringValue(value.toString)
      case StandardType.DayOfWeekType      => Value.StringValue(value.toString)
    }

  def toValue(input: DynamicValue): ResponseValue = {
    input match {
      case DynamicValue.Sequence(values)               => ResponseValue.ListValue(values.map(toValue).toList)
      case DynamicValue.Primitive(value, standardType) => toValue(value, standardType)
      case DynamicValue.Dictionary(_)                  => ???
      case DynamicValue.Singleton(_)                   => ???
      case DynamicValue.NoneValue                      => ???
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.SetValue(_)                    => ???
      case DynamicValue.Record(_, _)                   => ???
      case DynamicValue.Enumeration(_, _)              => ???
      case DynamicValue.RightValue(_)                  => ???
      case DynamicValue.SomeValue(_)                   => ???
      case DynamicValue.Tuple(_, _)                    => ???
      case DynamicValue.LeftValue(_)                   => ???
      case DynamicValue.Error(_)                       => ???
    }
  }
}
