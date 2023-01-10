package tailcall.gateway

import tailcall.gateway.Validation._
import zio.test.Assertion._
import zio.test._

object ValidationSpec extends ZIOSpecDefault {
  val isNumber: Validation[String, Int] = Validation.make[String] { s =>
    if (s.matches("^[0-9]+$"))
      Status.value(s.toInt)
    else
      Status.trace(s"Invalid number: $s")
  }

  val areAlphabets: Validation[String, String] = Validation.make[String] { s =>
    if (s.matches("^[A-Za-z]+$"))
      Status.value(s)
    else
      Status.trace(s"Invalid alphabets: $s")
  }

  def spec =
    suite("ValidationSpec")(
      test("test input with valid number") {
        assert(isNumber.validate("123").values)(equalTo(List(123)))
      },
      test("test input with valid string") {
        assert(areAlphabets.validate("abc").values)(equalTo(List("abc")))
      },
      test("test input with invalid number and string") {
        assert(isNumber.validate("123abc").traces)(equalTo(List("Invalid number: 123abc")))
      },
      test("test input with invalid string") {
        assert(areAlphabets.validate("123abc").traces)(equalTo(List("Invalid alphabets: 123abc")))
      },
      test("test empty input") {
        assert(isNumber.validate("").traces)(equalTo(List("Invalid number: ")))
      },
      test("test compose invalid input") {
        val composed = isNumber ++ areAlphabets
        assert(composed.validate("123abc").traces)(
          equalTo(List("Invalid number: 123abc", "Invalid alphabets: 123abc")),
        )
      },
      test("test compose validation") {
        val composed = isNumber ++ areAlphabets
        val result   = composed.validate("123")
        assert(result.traces)(equalTo(List("Invalid alphabets: 123"))) &&
        assert(result.values)(equalTo(List(123)))
      },
      test("test flatMap validation") {
        val flatMapped = isNumber.flatMap(_ => areAlphabets)
        assert(flatMapped.validate("123abc").traces)(equalTo(List("Invalid number: 123abc")))
      },
      test("test map validation") {
        val mapped = isNumber.map(res => res + 1)
        assert(mapped.validate("123").values)(equalTo(List(124)))
      },
    )
}
