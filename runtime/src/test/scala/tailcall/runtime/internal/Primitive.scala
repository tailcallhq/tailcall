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
    Gen.unit.map(value => Primitive.Value(value, StandardType.UnitType)),
    Gen.string.map(value => Primitive.Value(value, StandardType.StringType)),
    Gen.boolean.map(value => Primitive.Value(value, StandardType.BoolType)),
    Gen.byte.map(value => Primitive.Value(value, StandardType.ByteType)),
    Gen.short.map(value => Primitive.Value(value, StandardType.ShortType)),
    Gen.int.map(value => Primitive.Value(value, StandardType.IntType)),
    Gen.long.map(value => Primitive.Value(value, StandardType.LongType)),
    Gen.float.map(value => Primitive.Value(value, StandardType.FloatType)),
    Gen.double.map(value => Primitive.Value(value, StandardType.DoubleType)),
    Gen.chunkOf(Gen.byte).map(value => Primitive.Value(value, StandardType.BinaryType)),
    Gen.char.map(value => Primitive.Value(value, StandardType.CharType)),
    Gen.uuid.map(value => Primitive.Value(value, StandardType.UUIDType)),
    bigDecimalGen.map(value => Primitive.Value(value, StandardType.BigDecimalType)),
    bigIntegerGen.map(value => Primitive.Value(value, StandardType.BigIntegerType)),
    Gen.dayOfWeek.map(value => Primitive.Value(value, StandardType.DayOfWeekType)),
    Gen.month.map(value => Primitive.Value(value, StandardType.MonthType)),
    Gen.monthDay.map(value => Primitive.Value(value, StandardType.MonthDayType)),
    Gen.period.map(value => Primitive.Value(value, StandardType.PeriodType)),
    Gen.year.map(value => Primitive.Value(value, StandardType.YearType)),
    Gen.yearMonth.map(value => Primitive.Value(value, StandardType.YearMonthType)),
    Gen.zoneId.map(value => Primitive.Value(value, StandardType.ZoneIdType)),
    Gen.zoneOffset.map(value => Primitive.Value(value, StandardType.ZoneOffsetType)),
    Gen.finiteDuration.map(value => Primitive.Value(value, StandardType.DurationType)),
    Gen.instant.map(value => Primitive.Value(value, StandardType.InstantType)),
    Gen.localDate.map(value => Primitive.Value(value, StandardType.LocalDateType)),
    Gen.localTime.map(value => Primitive.Value(value, StandardType.LocalTimeType)),
    Gen.localDateTime.map(value => Primitive.Value(value, StandardType.LocalDateTimeType)),
    Gen.offsetTime.map(value => Primitive.Value(value, StandardType.OffsetTimeType)),
    Gen.offsetDateTime.map(value => Primitive.Value(value, StandardType.OffsetDateTimeType)),
    Gen.zonedDateTime.map(value => Primitive.Value(value, StandardType.ZonedDateTimeType))
  )
}
