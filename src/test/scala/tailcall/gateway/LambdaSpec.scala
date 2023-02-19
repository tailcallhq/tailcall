package tailcall.gateway

import tailcall.gateway.lambda._
import tailcall.gateway.remote.{EvaluationContext, LambdaRuntime, Lambda}
import zio.test.Assertion._
import zio.test._

object LambdaSpec extends ZIOSpecDefault {
  import tailcall.gateway.remote.Lambda._
  def spec =
    suite("lambda")(
      test("literal") {
        val l = Lambda("x").evaluate(1)
        assertZIO(l)(equalTo("x"))
      },
      suite("fromFunction")(
        test("fromFunction") {
          val program = Lambda.fromFunction[Int, Int](x => x + Lambda(1))
          assertZIO(program.evaluateWith(1))(equalTo(2))
        },
        test("apply") {
          val f1      = Lambda.fromFunction[Int, Int](x => x + Lambda(1))
          val program = f1(1)
          assertZIO(program.evaluateWith {})(equalTo(2))
        },
        test("higher order function") {
          val f1      = Lambda.fromFunction[Int ~> Int, Int](f => f.flatten(1))
          val program = f1(Lambda.fromFunction[Int, Int](x => x + Lambda(1)))

          assertZIO(program.evaluateWith {})(equalTo(2))
        }
      ),
      test("add") {
        val program = Lambda(1) + Lambda(2)
        assertZIO(program.evaluate(()))(equalTo(3))
      }
    ).provide(EvaluationContext.live, LambdaRuntime.live)
}
