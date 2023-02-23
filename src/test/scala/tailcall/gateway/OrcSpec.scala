package tailcall.gateway

import tailcall.gateway.ast.{Orc, TGraph}
import tailcall.gateway.service.{DynamicRuntime, EvaluationContext}
import zio.test.Assertion._
import zio.test._

object OrcSpec extends ZIOSpecDefault {
  def execute(graph: TGraph)(query: String) =
    graph.toGraphQL.interpreter.flatMap(_.execute(query, skipValidation = true)).map(_.data.toString())

  def spec =
    suite("OrcSpec")(
      test("one level") {
        val orc = Orc.obj("Query")("foo" -> Orc.value(100), "bar" -> Orc.value("BAR"))

        val response = execute(TGraph(orc))("""query {foo bar}""")
        assertZIO(response)(equalTo("{\"foo\":100,\"bar\":\"BAR\"}"))
      },
      test("two level") {
        val foo: Orc = Orc.obj("Foo")("value" -> Orc.value("foo"), "bar" -> Orc.ref("Bar"))
        val bar: Orc = Orc.obj("Bar")("value" -> Orc.value("bar"), "foo" -> Orc.ref("Foo"))
        val tGraph   = TGraph(foo, bar)
        val response = execute(tGraph)("{bar {foo {bar {foo {bar {value}}}}}}")

        assertZIO(response)(equalTo("{\"bar\":{\"foo\":{\"bar\":{\"foo\":{\"bar\":{\"value\":\"bar\"}}}}}}"))
      }
    ).provide(DynamicRuntime.live, EvaluationContext.live)
}
