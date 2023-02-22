package tailcall.gateway

import tailcall.gateway.lambda.Lambda.{logic, math}
import tailcall.gateway.lambda.{EvaluationContext, Lambda, DynamicRuntime, ~>}
import zio.test.Assertion._
import zio.test._

object LambdaSpec extends ZIOSpecDefault {
  import tailcall.gateway.lambda.Numeric._

  def spec =
    suite("Lambda")(
      suite("math")(
        test("add") {
          val program = math.add(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(3))
        },
        test("subtract") {
          val program = math.sub(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(-1))
        },
        test("multiply") {
          val program = math.mul(Lambda(2), Lambda(3))
          assertZIO(program.evaluate())(equalTo(6))
        },
        test("divide") {
          val program = math.div(Lambda(6), Lambda(3))
          assertZIO(program.evaluate())(equalTo(2))
        },
        test("modulo") {
          val program = math.mod(Lambda(7), Lambda(3))
          assertZIO(program.evaluate())(equalTo(1))
        },
        test("greater than") {
          val program = math.gt(Lambda(2), Lambda(1))
          assertZIO(program.evaluate())(isTrue)
        }
      ),
      suite("logical")(
        test("and") {
          val program = logic.and(Lambda(true), Lambda(true))
          assertZIO(program.evaluate())(isTrue)
        },
        test("or") {
          val program = logic.or(Lambda(true), Lambda(false))
          assertZIO(program.evaluate())(isTrue)
        },
        test("not") {
          val program = logic.not(Lambda(true))
          assertZIO(program.evaluate())(isFalse)
        },
        test("equal") {
          val program = logic.eq(Lambda(1), Lambda(1))
          assertZIO(program.evaluate())(equalTo(true))
        },
        test("not equal") {
          val program = logic.eq(Lambda(1), Lambda(2))
          assertZIO(program.evaluate())(equalTo(false))
        }
      ),
      suite("diverge")(
        test("isTrue") {
          val program = logic.cond(Lambda(true))(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate())(equalTo("Yes"))
        },
        test("isFalse") {
          val program = logic.cond(Lambda(false))(Lambda("Yes"), Lambda("No"))
          assertZIO(program.evaluate())(equalTo("No"))
        }
      ),
      suite("fromFunction")(
        test("one level") {
          val program = Lambda.fromLambdaFunction[Int, Int](i => math.add(i, Lambda(1)))
          assertZIO(program.evaluate(1))(equalTo(2))
        },
        test("two level") {
          val program = Lambda.fromLambdaFunction[Int, Int] { i =>
            val f1 = Lambda.fromLambdaFunction[Int, Int](j => math.mul(i, j))
            math.add(i, Lambda(1)) >>> f1
          }
          assertZIO(program.evaluate(10))(equalTo(110))
        },
        test("three level") {
          val program = Lambda.fromLambdaFunction[Int, Int] { i =>
            val f1 = Lambda.fromLambdaFunction[Int, Int] { j =>
              val f2 = Lambda.fromLambdaFunction[Int, Int](k => math.mul(math.mul(i, j), k))
              math.add(j, Lambda(1)) >>> f2
            }
            math.add(i, Lambda(1)) >>> f1
          }
          assertZIO(program.evaluate(10))(equalTo(10 * 11 * 12))
        },
        test("nested siblings") {
          val program = Lambda.fromLambdaFunction[Int, Int] { i =>
            val f1 = Lambda.fromLambdaFunction[Int, Int](j => math.mul(i, j))
            val f2 = Lambda.fromLambdaFunction[Int, Int](j => math.mul(i, j))
            math.add(math.add(i, Lambda(1)) >>> f1, math.sub(i, Lambda(1)) >>> f2)
          }
          assertZIO(program.evaluate(10))(equalTo(200))
        }
      ),
      suite("recursion")(
        test("sum") {
          val sum: Int ~> Int = Lambda.recurse[Int, Int] { next =>
            logic.cond(logic.eq(Lambda.identity[Int], Lambda(0)))(
              isTrue = Lambda(0),
              isFalse = math.add(Lambda.identity[Int], math.dec(Lambda.identity[Int]) >>> next)
            )
          }
          assertZIO(sum.evaluate(5))(equalTo(15))

        },
        test("factorial") {
          val factorial: Int ~> Int = Lambda.recurse[Int, Int](next =>
            logic.cond(math.gte(Lambda.identity[Int], Lambda(1)))(
              math.mul(Lambda.identity[Int], math.sub(Lambda.identity[Int], Lambda(1)) >>> next),
              Lambda(1)
            )
          )
          assertZIO(factorial.evaluate(5))(equalTo(120))
        },
        test("fibonnaci") {
          val fib = Lambda.recurse[Int, Int] { next =>
            logic.cond(math.gte(Lambda.identity[Int], Lambda(2)))(
              math.add(
                math.sub(Lambda.identity[Int], Lambda(1)) >>> next,
                math.sub(Lambda.identity[Int], Lambda(2)) >>> next
              ),
              Lambda.identity[Int]
            )
          }
          assertZIO(fib.evaluate(10))(equalTo(55))
        }
      )
    ).provide(DynamicRuntime.live, EvaluationContext.live)
}
