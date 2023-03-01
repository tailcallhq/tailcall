package tailcall.gateway.internal

import caliban.{InputValue, ResponseValue, Value}
import zio.schema.{DynamicValue, Schema, StandardType}

object DynamicValueUtil {
  def toValue(value: Any, standardType: StandardType[_]): Value =
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
      case DynamicValue.NoneValue                      => Value.NullValue
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.SetValue(_)                    => ???
      case DynamicValue.Record(_, _)                   => ???
      case DynamicValue.Enumeration(_, _)              => ???
      case DynamicValue.RightValue(_)                  => ???
      case DynamicValue.SomeValue(input)               => toValue(input)
      case DynamicValue.Tuple(_, _)                    => ???
      case DynamicValue.LeftValue(_)                   => ???
      case DynamicValue.Error(_)                       => ???
    }
  }

  def toInputValue(input: DynamicValue): InputValue = {
    input match {
      case DynamicValue.Sequence(values)               => InputValue.ListValue(values.map(toInputValue).toList)
      case DynamicValue.Primitive(value, standardType) => toValue(value, standardType)
      case DynamicValue.Dictionary(_)                  => ???
      case DynamicValue.Singleton(_)                   => ???
      case DynamicValue.NoneValue                      => ???
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.SetValue(_)                    => ???
      case DynamicValue.Record(_, b)      => InputValue.ObjectValue(b.map { case (k, v) => k -> toInputValue(v) })
      case DynamicValue.Enumeration(_, _) => ???
      case DynamicValue.RightValue(_)     => ???
      case DynamicValue.SomeValue(_)      => ???
      case DynamicValue.Tuple(_, _)       => ???
      case DynamicValue.LeftValue(_)      => ???
      case DynamicValue.Error(_)          => ???
    }
  }

  def as[A](d: DynamicValue)(implicit schema: Schema[A]): Option[A] = d.toTypedValueOption(schema)

  def getPath(d: DynamicValue, path: List[String]): Option[DynamicValue] =
    path match {
      case Nil          => Some(d)
      case head :: tail => d match {
          case DynamicValue.Record(_, b)  => b.get(head).flatMap(getPath(_, tail))
          case DynamicValue.SomeValue(a)  => getPath(a, path)
          case DynamicValue.Dictionary(b) =>
            val stringTag = StandardType.StringType.asInstanceOf[StandardType[Any]]
            b.collect { case (DynamicValue.Primitive(`head`, `stringTag`), value) => value }.headOption
              .flatMap(getPath(_, tail))
          case _                          => None
        }
    }

  // TODO: clean up
  def fromInputValue(input: InputValue): DynamicValue = {
    import caliban.InputValue.{ListValue, ObjectValue, VariableValue}
    import caliban.Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
    import caliban.Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
    import caliban.Value.{BooleanValue, EnumValue, NullValue, StringValue}

    input match {
      case ListValue(values)       => DynamicValue(values.map(fromInputValue(_)))
      case ObjectValue(fields)     => ???
      case StringValue(value)      => DynamicValue(value)
      case NullValue               => ???
      case BooleanValue(value)     => DynamicValue(value)
      case BigDecimalNumber(value) => DynamicValue(value)
      case DoubleNumber(value)     => DynamicValue(value)
      case FloatNumber(value)      => DynamicValue(value)
      case BigIntNumber(value)     => DynamicValue(value)
      case IntNumber(value)        => DynamicValue(value)
      case LongNumber(value)       => DynamicValue(value)
      case EnumValue(value)        => ???
      case VariableValue(name)     => ???
    }
  }
}
