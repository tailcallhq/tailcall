package tailcall.gateway

import tailcall.gateway.ast.Orc
import tailcall.gateway.lambda.{EvaluationContext, LambdaRuntime}
import zio.test.Assertion._
import zio.test._

object OrcSpec extends ZIOSpecDefault {
  def execute(orc: Orc)(query: String) =
    orc.toGraphQL.interpreter.flatMap(_.execute(query, skipValidation = true)).map(_.data.toString())

  def spec =
    suite("OrcSpec")(suite("execute")(
      test("one level") {
        val orc = Orc.obj("Query")("foo" -> Orc.value(100), "bar" -> Orc.value("BAR"))

        val response = execute(orc)("""query {foo bar}""")
        assertZIO(response)(equalTo("{\"foo\":100,\"bar\":\"BAR\"}"))
      }
      // test("two level") {
      //   def foo =
      //     Orc.obj("Foo")("value" -> Orc.value("foo"), "bar" -> Orc.ref("Bar"))

      //   def bar =
      //     Orc.obj("Bar")("value" -> Orc.value("bar"), "foo" -> Orc.ref("Foo"))

      //   val response = execute(foo)("{bar {foo {bar {foo {bar {value}}}}}}")

      //   assertZIO(response)(equalTo(
      //     "{\"bar\":{\"foo\":{\"bar\":{\"foo\":{\"bar\":{\"value\":\"bar\"}}}}}}"
      //   ))
      // }
//      test("list") {
//        val orc = Orc.make(
//          Orc.node("Query")(
//            "foo"             -> Resolver
//              .value {
//                OExit.value(Remote.fromSeq(Seq(
//                  Remote.record("a" -> Remote.dynamicValue("v1")),
//                  Remote.record("a" -> Remote.dynamicValue("v2"))
//                )))
//              }
//              .asList("Foo")
//          ),
//          Orc.node("Foo")("b" -> Resolver.value("foo"))
//        )
//
//        val response = execute(orc)("{foo {a b}}")
//        assertZIO(response)(equalTo("{\"foo\":[\"foo1\",\"foo2\"]}"))
//      }
    )).provide(LambdaRuntime.live, EvaluationContext.live)
}
