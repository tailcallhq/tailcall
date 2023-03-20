package tailcall.runtime.transcoder.value

import caliban.Value
import tailcall.runtime.internal.TValid
import zio.Chunk
import zio.schema.StandardType

trait ToValue {
  final def toValue[A](value: A, standardType: StandardType[A]): TValid[Nothing, Value] =
    TValid.succeed {
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
        case StandardType.BigDecimalType     => Value.FloatValue(BigDecimal(value.toString))
        case StandardType.YearType           => Value.IntValue(value.toString.toInt)
        case StandardType.ByteType           => Value.IntValue(value.toString.toByte)
        case StandardType.UUIDType           => Value.StringValue(value.toString)
        case StandardType.PeriodType         => Value.StringValue(value.toString)
        case StandardType.LongType           => Value.IntValue(value.toString.toLong)
        case StandardType.ZoneOffsetType     => Value.StringValue(value.toString)
        case StandardType.BigIntegerType     => Value.IntValue(BigInt(value.toString))
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
            .StringValue(java.util.Base64.getEncoder.encodeToString(value.asInstanceOf[Chunk[Byte]].toArray))
        case StandardType.DurationType       => Value.StringValue(value.toString)
        case StandardType.DayOfWeekType      => Value.StringValue(value.toString)
      }
    }
}
