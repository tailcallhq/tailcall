package tailcall.runtime.internal

import caliban.{InputValue, Value}
import zio.test.Gen

import java.math.BigInteger
import java.util.Random

object Caliban {
  val probablePrime = BigInt(BigInteger.probablePrime(100, new Random(0x9e3779b1L)))

  val genName   = Gen.string1(Gen.alphaChar)
  val genBigInt = Gen.bigInt(BigInt(0), probablePrime)
  val genBigNum = Gen.bigDecimal(BigDecimal(0), BigDecimal(probablePrime))

  val genIntValue: Gen[Any, Value.IntValue] = Gen.oneOf(
    Gen.int.map(Value.IntValue.IntNumber),
    Gen.long.map(Value.IntValue.LongNumber),
    genBigInt.map(Value.IntValue.BigIntNumber)
  )

  val genFloatValue: Gen[Any, Value.FloatValue] = Gen.oneOf(
    Gen.float.map(Value.FloatValue.FloatNumber),
    Gen.double.map(Value.FloatValue.DoubleNumber),
    genBigNum.map(Value.FloatValue.BigDecimalNumber)
  )

  val genValue: Gen[Any, Value] = Gen.oneOf(
    Gen.const(Value.NullValue),
    genIntValue,
    genFloatValue,
    Gen.string.map(Value.StringValue),
    Gen.boolean.map(Value.BooleanValue)
    // genName.map(Value.EnumValue)
  )

  val genInputValue: Gen[Any, InputValue] = Gen.suspend(Gen.oneOf(
    Gen.listOfBounded(0, 3)(genInputValue).map(InputValue.ListValue),
    Gen.mapOfBounded(0, 3)(genName, genInputValue).map(InputValue.ObjectValue),
    // genName.map(InputValue.VariableValue),
    genValue
  ))
}
