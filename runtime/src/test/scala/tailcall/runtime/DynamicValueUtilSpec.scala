package tailcall.runtime

import caliban.Value
import tailcall.runtime.internal.DynamicValueUtil._
import zio.schema.{DynamicValue, StandardType}
import zio.test._

import java.math.{BigDecimal, BigInteger}
import java.time._
import java.util.{Random, UUID}

object DynamicValueUtilSpec extends ZIOSpecDefault {
  val helloWorld           = "Hello World!"
  val meaningOfLife        = 42
  val myBirthMonth         = Month.SEPTEMBER
  val myBirthYear          = Year.of(1992)
  val myBirthYearMonth     = YearMonth.of(myBirthYear.getValue, myBirthMonth)
  val myBirthDay           = MonthDay.of(myBirthMonth, 2)
  val myBirthDate          = LocalDate.of(myBirthYear.getValue, myBirthMonth, myBirthDay.getDayOfMonth)
  val myBirthTime          = LocalTime.of(20, 40)
  val myBirthDateTime      = LocalDateTime.of(myBirthDate, myBirthTime)
  val myBirthTimeZone      = ZoneOffset.ofHoursMinutes(5, 30)
  val myBirthDateTimeZone  = OffsetDateTime.of(myBirthDateTime, myBirthTimeZone)
  val myBirthTimeZoneId    = ZoneId.ofOffset("UTC", myBirthTimeZone)
  val avogadrosNumber      = BigDecimal.valueOf(6.0221408e+23)
  val randomUUID           = UUID.fromString("137f0cfc-b664-412b-a4ac-9086755ccce5")
  val halfLifeTitanium44   = Period.ofYears(63)
  val magicNumberHashing   = 0x9e3779b1L
  val probablePrime        = BigInteger.probablePrime(100, new Random(magicNumberHashing))
  val myBirthTimeWithZone  = OffsetTime.of(myBirthTime, myBirthTimeZone)
  val epoch                = Instant.EPOCH
  val myBirthZonedDateTime = ZonedDateTime.of(myBirthDateTime, myBirthTimeZoneId)
  val binaryHelloWorld     = helloWorld.getBytes()
  val halfLifeActinium225  = Duration.ofDays(10)
  val myBirthDayOfWeek     = DayOfWeek.WEDNESDAY;

  override def spec =
    suite("DynamicValueUtilSpec")(
      test("asString") {
        assertTrue(asString(DynamicValue(helloWorld)) == Some("Hello World!")) &&
        assertTrue(asString(DynamicValue(meaningOfLife)) == Some("42")) &&
        assertTrue(asString(DynamicValue(List(meaningOfLife))) == None)
      },
      test("toValue") {
        assertTrue(toValue(helloWorld, StandardType.StringType) == Value.StringValue("Hello World!")) &&
        assertTrue(toValue(Int.MaxValue, StandardType.IntType) == Value.IntValue(2147483647)) &&
        assertTrue(toValue(myBirthDay, StandardType.MonthDayType) == Value.StringValue("--09-02")) &&
        assertTrue(toValue(myBirthDateTime, StandardType.LocalDateTimeType) == Value.StringValue("1992-09-02T20:40")) &&
        assertTrue(toValue(true, StandardType.BoolType) == Value.BooleanValue(true)) &&
        assertTrue(toValue(myBirthTime, StandardType.LocalTimeType) == Value.StringValue("20:40")) &&
        assertTrue(
          toValue(myBirthDateTimeZone, StandardType.OffsetDateTimeType) == Value.StringValue("1992-09-02T20:40+05:30")
        ) &&
        assertTrue(toValue(myBirthMonth, StandardType.MonthType) == Value.StringValue("SEPTEMBER")) &&
        assertTrue(toValue(Short.MaxValue, StandardType.ShortType) == Value.IntValue(32767)) &&
        assertTrue(toValue(myBirthTimeZoneId, StandardType.ZoneIdType) == Value.StringValue("UTC+05:30")) &&
        assertTrue(toValue(avogadrosNumber, StandardType.BigDecimalType) == Value.StringValue("6.0221408E+23")) &&
        assertTrue(toValue(myBirthYear, StandardType.YearType) == Value.IntValue(1992)) &&
        assertTrue(toValue(Byte.MaxValue, StandardType.ByteType) == Value.IntValue(127)) &&
        assertTrue(
          toValue(randomUUID, StandardType.UUIDType) == Value.StringValue("137f0cfc-b664-412b-a4ac-9086755ccce5")
        ) &&
        assertTrue(toValue(halfLifeTitanium44, StandardType.PeriodType) == Value.StringValue("P63Y")) &&
        assertTrue(toValue(Long.MaxValue, StandardType.LongType) == Value.StringValue("9223372036854775807")) &&
        assertTrue(toValue(myBirthTimeZone, StandardType.ZoneOffsetType) == Value.StringValue("+05:30")) &&
        assertTrue(
          toValue(probablePrime, StandardType.BigIntegerType) == Value.StringValue("799058976649937674302168095891")
        ) &&
        assertTrue(toValue(myBirthTimeWithZone, StandardType.OffsetTimeType) == Value.StringValue("20:40+05:30")) &&
        assertTrue(toValue((), StandardType.UnitType) == Value.NullValue) &&
        assertTrue(toValue(Double.MaxValue, StandardType.DoubleType) == Value.FloatValue(1.7976931348623157e308)) &&
        assertTrue(toValue(epoch, StandardType.InstantType) == Value.StringValue("1970-01-01T00:00:00Z")) &&
        assertTrue(toValue(Float.MaxValue, StandardType.FloatType) == Value.FloatValue(3.4028235e38.toFloat)) &&
        assertTrue(toValue(myBirthDate, StandardType.LocalDateType) == Value.StringValue("1992-09-02")) &&
        assertTrue(
          toValue(myBirthZonedDateTime, StandardType.ZonedDateTimeType) == Value
            .StringValue("1992-09-02T20:40+05:30[UTC+05:30]")
        ) &&
        assertTrue(toValue(myBirthYearMonth, StandardType.YearMonthType) == Value.StringValue("1992-09")) &&
        assertTrue(toValue(Char.MaxValue, StandardType.CharType) == Value.StringValue("￿")) &&
        assertTrue(toValue(binaryHelloWorld, StandardType.BinaryType) == Value.StringValue("SGVsbG8gV29ybGQh")) &&
        assertTrue(toValue(halfLifeActinium225, StandardType.DurationType) == Value.StringValue("PT240H")) &&
        assertTrue(toValue(myBirthDayOfWeek, StandardType.DayOfWeekType) == Value.StringValue("WEDNESDAY"))
      }
    )
}
