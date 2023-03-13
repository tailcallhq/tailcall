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

  private val hashMagicNumber = 0x9e3779b1L
  private val probablePrime   = BigInteger.probablePrime(100, new Random(hashMagicNumber))
  private val bigDecimalGen   = Gen.bigDecimalJava(BigDecimal.ZERO, new BigDecimal(probablePrime))
  private val bigIntegerGen   = Gen.bigIntegerJava(BigInteger.ZERO, probablePrime)

  val gen: Gen[Any, Primitive] = Gen.oneOf(
    Gen.unit.map(value => Value(value, StandardType.UnitType)),
    Gen.string.map(value => Value(value, StandardType.StringType)),
    Gen.boolean.map(value => Value(value, StandardType.BoolType)),
    Gen.byte.map(value => Value(value, StandardType.ByteType)),
    Gen.short.map(value => Value(value, StandardType.ShortType)),
    Gen.int.map(value => Value(value, StandardType.IntType)),
    Gen.long.map(value => Value(value, StandardType.LongType)),
    Gen.float.map(value => Value(value, StandardType.FloatType)),
    Gen.double.map(value => Value(value, StandardType.DoubleType)),
    Gen.chunkOf(Gen.byte).map(value => Value(value, StandardType.BinaryType)),
    Gen.char.map(value => Value(value, StandardType.CharType)),
    Gen.uuid.map(value => Value(value, StandardType.UUIDType)),
    bigDecimalGen.map(value => Value(value, StandardType.BigDecimalType)),
    bigIntegerGen.map(value => Value(value, StandardType.BigIntegerType)),
    Gen.dayOfWeek.map(value => Value(value, StandardType.DayOfWeekType)),
    Gen.month.map(value => Value(value, StandardType.MonthType)),
    Gen.monthDay.map(value => Value(value, StandardType.MonthDayType)),
    Gen.period.map(value => Value(value, StandardType.PeriodType)),
    Gen.year.map(value => Value(value, StandardType.YearType)),
    Gen.yearMonth.map(value => Value(value, StandardType.YearMonthType)),
    Gen.zoneId.map(value => Value(value, StandardType.ZoneIdType)),
    Gen.zoneOffset.map(value => Value(value, StandardType.ZoneOffsetType)),
    Gen.finiteDuration.map(value => Value(value, StandardType.DurationType)),
    Gen.instant.map(value => Value(value, StandardType.InstantType)),
    Gen.localDate.map(value => Value(value, StandardType.LocalDateType)),
    Gen.localTime.map(value => Value(value, StandardType.LocalTimeType)),
    Gen.localDateTime.map(value => Value(value, StandardType.LocalDateTimeType)),
    Gen.offsetTime.map(value => Value(value, StandardType.OffsetTimeType)),
    Gen.offsetDateTime.map(value => Value(value, StandardType.OffsetDateTimeType)),
    Gen.zonedDateTime.map(value => Value(value, StandardType.ZonedDateTimeType))
  )
}
