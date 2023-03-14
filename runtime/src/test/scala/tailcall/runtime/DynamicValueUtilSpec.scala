package tailcall.runtime

import caliban.{InputValue, ResponseValue, Value}
import tailcall.runtime.internal.DynamicValueUtil._
import tailcall.runtime.internal.Primitive
import zio.json.ast.Json
import zio.schema.{DeriveSchema, DynamicValue, Schema, TypeId}
import zio.test._

import java.math.{BigDecimal, BigInteger}
import java.util.Random
import scala.collection.immutable.ListMap
import scala.util.Try

object DynamicValueUtilSpec extends ZIOSpecDefault {
  val helloWorld         = "Hello World!"
  val meaningOfLife      = 42
  val avogadrosNumber    = BigDecimal.valueOf(6.0221408e+23)
  val magicNumberHashing = 0x9e3779b1L
  val probablePrime      = BigInteger.probablePrime(100, new Random(magicNumberHashing))

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
        check(Primitive.gen) { primitive =>
          assertTrue(asString(primitive.toDynamicValue) == Some(primitive.value.toString))
        } &&
        assertTrue(asString(DynamicValue(List(meaningOfLife))) == None)
      },
      test("toResponseValue") {
        assertTrue(
          toResponseValue(DynamicValue(Foobar(List(meaningOfLife), DynamicValue(())))) == ResponseValue
            .ObjectValue(List("foo" -> ResponseValue.ListValue(List(Value.IntValue(42))), "bar" -> Value.NullValue))
        ) &&
        assertTrue(
          toResponseValue(
            DynamicValue(Map("foo" -> DynamicValue(List(meaningOfLife)), "bar" -> DynamicValue(())))
          ) == ResponseValue
            .ObjectValue(List("foo" -> ResponseValue.ListValue(List(Value.IntValue(42))), "bar" -> Value.NullValue))
        ) &&
        assert(Try(toResponseValue(DynamicValue(Map(meaningOfLife -> helloWorld)))))(Assertion.isFailure(
          Assertion.hasMessage(Assertion.equalTo("could not transform"))
        ))
      },
      test("toInputValue") {
        assertTrue(
          toInputValue(DynamicValue(Foobaz(List(meaningOfLife), List()))) == InputValue.ObjectValue(
            Map("foo" -> InputValue.ListValue(List(Value.IntValue(42))), "baz" -> InputValue.ListValue(List()))
          )
        ) &&
        assertTrue(
          toInputValue(DynamicValue(Map("foo" -> List(meaningOfLife), "bar" -> List()))) == InputValue.ObjectValue(
            Map("foo" -> InputValue.ListValue(List(Value.IntValue(42))), "bar" -> InputValue.ListValue(List()))
          )
        ) &&
        assert(Try(toInputValue(DynamicValue(Map(meaningOfLife -> helloWorld)))))(Assertion.isFailure(
          Assertion.hasMessage(Assertion.equalTo("could not transform"))
        ))
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
          fromInputValue(InputValue.ObjectValue(Map(
            "foo" -> InputValue.ListValue(List(
              Value.StringValue(helloWorld),
              Value.BooleanValue(true),
              Value.FloatValue.BigDecimalNumber(avogadrosNumber),
              Value.FloatValue.DoubleNumber(Double.MaxValue),
              Value.FloatValue.FloatNumber(Float.MaxValue),
              Value.IntValue.BigIntNumber(probablePrime),
              Value.IntValue.IntNumber(Int.MaxValue),
              Value.IntValue.LongNumber(Long.MaxValue)
            ))
          ))) == DynamicValue(Map(
            "foo" -> List(
              DynamicValue("Hello World!"),
              DynamicValue(true),
              DynamicValue(BigDecimal.valueOf(6.0221408e+23)),
              DynamicValue(1.7976931348623157e308),
              DynamicValue(3.4028235e38.toFloat),
              DynamicValue(new BigInteger("799058976649937674302168095891")),
              DynamicValue(2147483647),
              DynamicValue(9223372036854775807L)
            )
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
