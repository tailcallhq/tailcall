package tailcall.runtime.internal

import caliban.{InputValue, ResponseValue, Value}
import zio.test.Gen

object CalibanGen {
  val probablePrime = BigInt("799058976649937674302168095891")

  val genName   = Gen.string1(Gen.alphaChar)
  val genBigInt = Gen.bigInt(BigInt(0), probablePrime)
  val genBigNum = Gen.bigDecimal(BigDecimal(0), BigDecimal(probablePrime))

  val genIntValue: Gen[Any, Value.IntValue] = Gen.oneOf(
    Gen.int.map(Value.IntValue.IntNumber(_)),
    Gen.long.map(Value.IntValue.LongNumber(_)),
    genBigInt.map(Value.IntValue.BigIntNumber(_)),
  )

  val genFloatValue: Gen[Any, Value.FloatValue] = Gen.oneOf(
    Gen.float.map(Value.FloatValue.FloatNumber(_)),
    Gen.double.map(Value.FloatValue.DoubleNumber(_)),
    genBigNum.map(Value.FloatValue.BigDecimalNumber(_)),
  )

  val genValue: Gen[Any, Value] = Gen.oneOf(
    Gen.const(Value.NullValue),
    genIntValue,
    genFloatValue,
    Gen.string.map(Value.StringValue(_)),
    Gen.boolean.map(Value.BooleanValue(_)),
  )

  val genInputValue: Gen[Any, InputValue] = Gen.suspend(Gen.oneOf(
    Gen.listOfBounded(0, 2)(genInputValue).map(InputValue.ListValue(_)),
    Gen.mapOfBounded(0, 2)(genName, genInputValue).map(InputValue.ObjectValue(_)),
    genValue,
  ))

  val genResponseValue: Gen[Any, ResponseValue] = Gen.suspend(Gen.oneOf(
    Gen.listOfBounded(0, 2)(genResponseValue).map(ResponseValue.ListValue(_)),
    Gen.listOfBounded(0, 2)(for {
      key   <- genName
      value <- genResponseValue
    } yield key -> value).map(ResponseValue.ObjectValue(_)),
    genValue,
  ))
}
