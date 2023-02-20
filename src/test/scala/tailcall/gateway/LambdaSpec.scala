package tailcall.gateway

import tailcall.gateway.lambda.{EvaluationContext, Lambda, LambdaRuntime}
import zio.test.Assertion._
import zio.test._

object LambdaSpec extends ZIOSpecDefault {
  import tailcall.gateway.lambda.Numeric._
  import tailcall.gateway.remote._

  def spec =
    suite("Lambda")(
      suite("math")(
        test("add") {
          val program = Lambda.math.add(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(3))
        },
        test("subtract") {
          val program = Lambda.math.subtract(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(-1))
        },
        test("multiply") {
          val program = Lambda.math.multiply(Lambda(2), Lambda(3))
          assertZIO(program.evaluate())(equalTo(6))
        },
        test("divide") {
          val program = Lambda.math.divide(Lambda(6), Lambda(3))
          assertZIO(program.evaluate())(equalTo(2))
        },
        test("modulo") {
          val program = Lambda.math.modulo(Lambda(7), Lambda(3))
          assertZIO(program.evaluate())(equalTo(1))
        },
        test("greater than") {
          val program = Lambda.math.gt(Lambda(2), Lambda(1))
          assertZIO(program.evaluate())(isTrue)
        }
      ),
      suite("logical")(
        test("and") {
          val program = Lambda.logic.and(Lambda(true), Lambda(true))
          assertZIO(program.evaluate())(isTrue)
        },
        test("or") {
          val program = Lambda.logic.or(Lambda(true), Lambda(false))
          assertZIO(program.evaluate())(isTrue)
        },
        test("not") {
          val program = Lambda.logic.not(Lambda(true))
          assertZIO(program.evaluate())(isFalse)
        },
        test("equal") {
          val program = Lambda.logic.equalTo(Lambda(1), Lambda(1))
          assertZIO(program.evaluate())(equalTo(true))
        },
        test("not equal") {
          val program = Lambda.logic.equalTo(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(false))
        }
      ),
      suite("diverge")(
        test("isTrue") {
          val program = Lambda.logic.diverge(Lambda(true), Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate())(equalTo("Yes"))
        },
        test("isFalse") {
          val program = Lambda.logic.diverge(Lambda(false), Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate())(equalTo("No"))
        }
      ),
      suite("fromFunction")(test("evaluate") {
        val program = Lambda.fromFunction[Int, Int](i => i + Remote(1))
        assertZIO(program.evaluate(1))(equalTo(2))
      })
    ).provide(LambdaRuntime.live, EvaluationContext.live)
}
