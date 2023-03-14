package tailcall.runtime.internal

import zio.schema.{DynamicValue, StandardType}
import zio.test.Gen

import java.math.{BigDecimal, BigInteger}
import java.util.Random

sealed trait Primitive {
  type Value
  val value: Value
  val standardType: StandardType[Value]
  val toDynamicValue = DynamicValue.Primitive(value, standardType)
}

object Primitive {
  final case class Value[A](value: A, standardType: StandardType[A]) extends Primitive {
    type Value = A
  }

  private val probablePrime = BigInteger.probablePrime(100, new Random(0x9e3779b1L))
  private val genBigDecimal = Gen.bigDecimalJava(BigDecimal.ZERO, new BigDecimal(probablePrime))
  private val genBigInteger = Gen.bigIntegerJava(BigInteger.ZERO, probablePrime)

  val gen: Gen[Any, Primitive] = Gen.oneOf(
    Gen.unit.map(Value(_, StandardType.UnitType)),
    Gen.string.map(Value(_, StandardType.StringType)),
    Gen.boolean.map(Value(_, StandardType.BoolType)),
    Gen.byte.map(Value(_, StandardType.ByteType)),
    Gen.short.map(Value(_, StandardType.ShortType)),
    Gen.int.map(Value(_, StandardType.IntType)),
    Gen.long.map(Value(_, StandardType.LongType)),
    Gen.float.map(Value(_, StandardType.FloatType)),
    Gen.double.map(Value(_, StandardType.DoubleType)),
    Gen.chunkOf(Gen.byte).map(Value(_, StandardType.BinaryType)),
    Gen.char.map(Value(_, StandardType.CharType)),
    Gen.uuid.map(Value(_, StandardType.UUIDType)),
    genBigDecimal.map(Value(_, StandardType.BigDecimalType)),
    genBigInteger.map(Value(_, StandardType.BigIntegerType)),
    Gen.dayOfWeek.map(Value(_, StandardType.DayOfWeekType)),
    Gen.month.map(Value(_, StandardType.MonthType)),
    Gen.monthDay.map(Value(_, StandardType.MonthDayType)),
    Gen.period.map(Value(_, StandardType.PeriodType)),
    Gen.year.map(Value(_, StandardType.YearType)),
    Gen.yearMonth.map(Value(_, StandardType.YearMonthType)),
    Gen.zoneId.map(Value(_, StandardType.ZoneIdType)),
    Gen.zoneOffset.map(Value(_, StandardType.ZoneOffsetType)),
    Gen.finiteDuration.map(Value(_, StandardType.DurationType)),
    Gen.instant.map(Value(_, StandardType.InstantType)),
    Gen.localDate.map(Value(_, StandardType.LocalDateType)),
    Gen.localTime.map(Value(_, StandardType.LocalTimeType)),
    Gen.localDateTime.map(Value(_, StandardType.LocalDateTimeType)),
    Gen.offsetTime.map(Value(_, StandardType.OffsetTimeType)),
    Gen.offsetDateTime.map(Value(_, StandardType.OffsetDateTimeType)),
    Gen.zonedDateTime.map(Value(_, StandardType.ZonedDateTimeType))
  )
}
