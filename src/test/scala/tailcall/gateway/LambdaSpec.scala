package tailcall.gateway

import tailcall.gateway.lambda.Lambda.math._
import tailcall.gateway.lambda.{EvaluationContext, Lambda, LambdaRuntime}
import zio.test.Assertion._
import zio.test._

object LambdaSpec extends ZIOSpecDefault {
  import tailcall.gateway.lambda.Numeric._

  def spec =
    suite("Lambda")(
      suite("math")(
        test("add") {
          val program = add(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(3))
        },
        test("subtract") {
          val program = subtract(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(-1))
        },
        test("multiply") {
          val program = multiply(Lambda(2), Lambda(3))
          assertZIO(program.evaluate())(equalTo(6))
        },
        test("divide") {
          val program = divide(Lambda(6), Lambda(3))
          assertZIO(program.evaluate())(equalTo(2))
        },
        test("modulo") {
          val program = modulo(Lambda(7), Lambda(3))
          assertZIO(program.evaluate())(equalTo(1))
        },
        test("greater than") {
          val program = gt(Lambda(2), Lambda(1))
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
      suite("fromFunction")(
        test("one level") {
          val program = Lambda.fromLambdaFunction[Int, Int](i => add(i, Lambda(1)))
          assertZIO(program.evaluate(1))(equalTo(2))
        },
        test("two level") {
          val program = Lambda.fromLambdaFunction[Int, Int] { i =>
            val f1 = Lambda.fromLambdaFunction[Int, Int](j => multiply(i, j))
            add(i, Lambda(1)) >>> f1
          }
          assertZIO(program.evaluate(10))(equalTo(110))
        },
        test("three level") {
          val program = Lambda.fromLambdaFunction[Int, Int] { i =>
            val f1 = Lambda.fromLambdaFunction[Int, Int] { j =>
              val f2 = Lambda.fromLambdaFunction[Int, Int](k => multiply(multiply(i, j), k))
              add(j, Lambda(1)) >>> f2
            }
            add(i, Lambda(1)) >>> f1
          }
          assertZIO(program.evaluate(10))(equalTo(10 * 11 * 12))
        },
        test("nested siblings") {
          val program = Lambda.fromLambdaFunction[Int, Int] { i =>
            val f1 = Lambda.fromLambdaFunction[Int, Int](j => multiply(i, j))
            val f2 = Lambda.fromLambdaFunction[Int, Int](j => multiply(i, j))
            add(add(i, Lambda(1)) >>> f1, subtract(i, Lambda(1)) >>> f2)

          }
          assertZIO(program.evaluate(10))(equalTo(200))
        }
      )
    ).provide(LambdaRuntime.live, EvaluationContext.live)
}
