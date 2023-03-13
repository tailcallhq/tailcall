package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.internal.DynamicValueUtil._
import zio.json.ast.Json
import zio.schema.{DeriveSchema, DynamicValue, Schema, StandardType, TypeId}
import zio.test._

import java.math.{BigDecimal, BigInteger}
import java.time._
import java.util.{Random, UUID}
import scala.collection.immutable.ListMap

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

  sealed trait Foo

  final case class Foobar(foo: List[Int], bar: DynamicValue)
  final case class Foobaz(foo: List[Int], baz: List[String])

  object Foo {
    final case class Bar(bar: List[Option[Int]])        extends Foo
    final case class Baz(baz: Set[Either[Int, String]]) extends Foo

    implicit val schema: Schema[Foo] = DeriveSchema.gen[Foo]
  }

  object Foobar {
    implicit val schema: Schema[Foobar] = DeriveSchema.gen[Foobar]
  }

  object Foobaz {
    implicit val schema: Schema[Foobaz] = DeriveSchema.gen[Foobaz]
  }

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
      },
      test("toResponseValue") {
        assertTrue(
          toResponseValue(DynamicValue(Foobar(List(meaningOfLife), DynamicValue(())))) == ResponseValue
            .ObjectValue(List(("foo", ResponseValue.ListValue(List(Value.IntValue(42)))), ("bar", Value.NullValue)))
        )
      },
      test("toInputValue") {
        assertTrue(
          toInputValue(DynamicValue(Foobaz(List(meaningOfLife), List()))) == InputValue.ObjectValue(
            Map("foo" -> InputValue.ListValue(List(Value.IntValue(42))), "baz" -> InputValue.ListValue(List()))
          )
        )
      },
      test("toTyped") {
        assertTrue(toTyped[String](DynamicValue(helloWorld)) == Some("Hello World!")) &&
        assertTrue(toTyped[String](DynamicValue(meaningOfLife)) == None)
      },
      test("getPath") {
        val d = DynamicValue(Foobar(List(meaningOfLife), DynamicValue(Option(Map("baz" -> helloWorld)))))
        assertTrue(getPath(d, List("foo", "0")) == Some(DynamicValue(42))) &&
        assertTrue(getPath(d, List("bar", "baz")) == Some(DynamicValue("Hello World!"))) &&
        assertTrue(getPath(d, List("foo", "1")) == None) &&
        assertTrue(getPath(d, List("bar", "qux")) == None) &&
        assertTrue(getPath(d, List("quux")) == None)
      },
      test("fromInputValue") {
        assertTrue(
          fromInputValue(InputValue.ListValue(List(
            Value.StringValue(helloWorld),
            Value.BooleanValue(true),
            Value.FloatValue.BigDecimalNumber(avogadrosNumber),
            Value.FloatValue.DoubleNumber(Double.MaxValue),
            Value.FloatValue.FloatNumber(Float.MaxValue),
            Value.IntValue.BigIntNumber(probablePrime),
            Value.IntValue.IntNumber(Int.MaxValue),
            Value.IntValue.LongNumber(Long.MaxValue)
          ))) == DynamicValue(List(
            DynamicValue("Hello World!"),
            DynamicValue(true),
            DynamicValue(BigDecimal.valueOf(6.0221408e+23)),
            DynamicValue(1.7976931348623157e308),
            DynamicValue(3.4028235e38.toFloat),
            DynamicValue(new BigInteger("799058976649937674302168095891")),
            DynamicValue(2147483647),
            DynamicValue(9223372036854775807L)
          ))
        )
      },
      test("record") {
        assertTrue(
          record("foo" -> DynamicValue(List(meaningOfLife)), "bar" -> DynamicValue(helloWorld)) == DynamicValue
            .Record(TypeId.Structural, ListMap("foo" -> DynamicValue(List(42)), "bar" -> DynamicValue("Hello World!")))
        )
      },
      test("fromJson") {
        assertTrue(
          fromJson(Json.Obj(
            "foo" -> Json.Arr(Json.Str(helloWorld), Json.Num(meaningOfLife), Json.Bool(true)),
            "bar" -> Json.Null
          )) == DynamicValue.Record(
            TypeId.Structural,
            ListMap(
              "foo" -> DynamicValue(
                List(DynamicValue("Hello World!"), DynamicValue(BigDecimal.valueOf(42L)), DynamicValue(true))
              ),
              "bar" -> DynamicValue.NoneValue
            )
          )
        )
      },
      test("toJsonPrimitive") {
        assertTrue(toJsonPrimitive((), StandardType.UnitType) == Json.Str("()")) &&
        assertTrue(toJsonPrimitive(helloWorld, StandardType.StringType) == Json.Str("Hello World!")) &&
        assertTrue(toJsonPrimitive(true, StandardType.BoolType) == Json.Bool(true)) &&
        assertTrue(toJsonPrimitive(Byte.MaxValue, StandardType.ByteType) == Json.Str("127")) &&
        assertTrue(toJsonPrimitive(Short.MaxValue, StandardType.ShortType) == Json.Str("32767")) &&
        assertTrue(toJsonPrimitive(Int.MaxValue, StandardType.IntType) == Json.Num(2147483647)) &&
        assertTrue(toJsonPrimitive(Long.MaxValue, StandardType.LongType) == Json.Num(9223372036854775807L)) &&
        assertTrue(toJsonPrimitive(Float.MaxValue, StandardType.FloatType) == Json.Num(3.4028234663852886e+38)) &&
        assertTrue(toJsonPrimitive(Double.MaxValue, StandardType.DoubleType) == Json.Num(1.7976931348623157e308)) &&
        assertTrue(toJsonPrimitive(binaryHelloWorld, StandardType.BinaryType) == Json.Str("SGVsbG8gV29ybGQh")) &&
        assertTrue(toJsonPrimitive(Char.MaxValue, StandardType.CharType) == Json.Str("￿")) &&
        assertTrue(
          toJsonPrimitive(randomUUID, StandardType.UUIDType) == Json.Str("137f0cfc-b664-412b-a4ac-9086755ccce5")
        ) &&
        assertTrue(toJsonPrimitive(avogadrosNumber, StandardType.BigDecimalType) == Json.Str("6.0221408E+23")) &&
        assertTrue(
          toJsonPrimitive(probablePrime, StandardType.BigIntegerType) == Json.Str("799058976649937674302168095891")
        ) &&
        assertTrue(toJsonPrimitive(myBirthDayOfWeek, StandardType.DayOfWeekType) == Json.Str("WEDNESDAY")) &&
        assertTrue(toJsonPrimitive(myBirthMonth, StandardType.MonthType) == Json.Str("SEPTEMBER")) &&
        assertTrue(toJsonPrimitive(myBirthDay, StandardType.MonthDayType) == Json.Str("--09-02")) &&
        assertTrue(toJsonPrimitive(halfLifeTitanium44, StandardType.PeriodType) == Json.Str("P63Y")) &&
        assertTrue(toJsonPrimitive(myBirthYear, StandardType.YearType) == Json.Str("1992")) &&
        assertTrue(toJsonPrimitive(myBirthYearMonth, StandardType.YearMonthType) == Json.Str("1992-09")) &&
        assertTrue(toJsonPrimitive(myBirthTimeZoneId, StandardType.ZoneIdType) == Json.Str("UTC+05:30")) &&
        assertTrue(toJsonPrimitive(myBirthTimeZone, StandardType.ZoneOffsetType) == Json.Str("+05:30")) &&
        assertTrue(toJsonPrimitive(halfLifeActinium225, StandardType.DurationType) == Json.Str("PT240H")) &&
        assertTrue(toJsonPrimitive(epoch, StandardType.InstantType) == Json.Str("1970-01-01T00:00:00Z")) &&
        assertTrue(toJsonPrimitive(myBirthDate, StandardType.LocalDateType) == Json.Str("1992-09-02")) &&
        assertTrue(toJsonPrimitive(myBirthTime, StandardType.LocalTimeType) == Json.Str("20:40")) &&
        assertTrue(toJsonPrimitive(myBirthDateTime, StandardType.LocalDateTimeType) == Json.Str("1992-09-02T20:40")) &&
        assertTrue(toJsonPrimitive(myBirthTimeWithZone, StandardType.OffsetTimeType) == Json.Str("20:40+05:30")) &&
        assertTrue(
          toJsonPrimitive(myBirthDateTimeZone, StandardType.OffsetDateTimeType) == Json.Str("1992-09-02T20:40+05:30")
        ) &&
        assertTrue(
          toJsonPrimitive(myBirthZonedDateTime, StandardType.ZonedDateTimeType) == Json
            .Str("1992-09-02T20:40+05:30[UTC+05:30]")
        )
      },
      test("toJson") {
        val bar: Foo = Foo.Bar(List(Some(meaningOfLife), None))
        val baz: Foo = Foo.Baz(Set(Left(meaningOfLife), Right(helloWorld)))
        assertTrue(
          toJson(DynamicValue((bar, baz))) == Json.Arr(
            Json.Obj("Bar" -> Json.Obj("bar" -> Json.Arr(Json.Num(42), Json.Null))),
            Json.Obj("Baz" -> Json.Obj("baz" -> Json.Arr(Json.Num(42), Json.Str("Hello World!"))))
          )
        )
      }
    )
}
