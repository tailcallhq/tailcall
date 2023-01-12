package tailcall.gateway

import zio.test.Assertion._
import zio.test._

object ValidationSpec extends ZIOSpecDefault {
  val isNumber: Validation[Any, Nothing, String, Int] =
    for {
      s      <- Validation.access[String]
      result <-
        if (s.matches("^[0-9]+$"))
          Validation.value(s.toInt)
        else
          Validation.trace(s"Invalid number: $s")
    } yield result

  val areAlphabets: Validation[Any, Nothing, String, String] =
    for {
      s      <- Validation.access[String]
      result <-
        if (s.matches("^[A-Za-z]+$"))
          Validation.value(s)
        else
          Validation.trace(s"Invalid alphabets: $s")
    } yield result

  def spec =
    suite("ValidationSpec")(
      test("test input with valid number") {
        assertZIO(isNumber.eval.values("123"))(equalTo(List(123)))
      },
      test("test input with valid string") {
        assertZIO(areAlphabets.eval.values("abc"))(equalTo(List("abc")))
      },
      test("test input with invalid number and string") {
        assertZIO(isNumber.eval.traces("123abc"))(equalTo(List("Invalid number: 123abc")))
      },
      test("test input with invalid string") {
        assertZIO(areAlphabets.eval.traces("123abc"))(equalTo(List("Invalid alphabets: 123abc")))
      },
      test("test empty input") {
        assertZIO(isNumber.eval.traces(""))(equalTo(List("Invalid number: ")))
      },
      test("test compose invalid input") {
        val composed = isNumber ++ areAlphabets
        assertZIO(composed.eval.traces("123abc"))(
          equalTo(List("Invalid number: 123abc", "Invalid alphabets: 123abc")),
        )
      },
      test("test compose validation") {
        val composed = isNumber ++ areAlphabets
        val result   = composed.eval
        assertZIO(result.traces("123"))(equalTo(List("Invalid alphabets: 123"))) &&
        assertZIO(result.values("123"))(equalTo(List(123)))
      },
      test("test flatMap validation") {
        val flatMapped = isNumber.flatMap(_ => areAlphabets)
        assertZIO(flatMapped.eval.traces("123abc"))(equalTo(List("Invalid number: 123abc")))
      },
      test("test flatMap validation") {
        val flatMapped = isNumber.flatMap(_ => areAlphabets)
        assertZIO(flatMapped.eval.traces("123"))(equalTo(List("Invalid alphabets: 123")))
      },
      test("test map validation") {
        val mapped = isNumber.map(res => res + 1)
        assertZIO(mapped.eval.values("123"))(equalTo(List(124)))
      },
    )
}
