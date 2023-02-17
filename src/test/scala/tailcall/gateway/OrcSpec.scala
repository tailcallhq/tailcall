package tailcall.gateway

import tailcall.gateway.ast.Orc
import tailcall.gateway.ast.Orc.Resolver
import tailcall.gateway.remote.{EvaluationContext, RemoteRuntime}
import zio.test.Assertion._
import zio.test._

object OrcSpec extends ZIOSpecDefault {
  def execute(orc: Orc)(query: String) =
    orc
      .toGraphQL
      .interpreter
      .flatMap(_.execute(query, skipValidation = true))
      .map(_.data.toString())

  def spec =
    suite("OrcSpec")(suite("execute")(
      test("one level") {
        val orc = Orc.make {
          Orc.node("Query")(
            "foo" -> Resolver.value(100),
            "bar" -> Resolver.value("BAR")
          )
        }

        val response = execute(orc)("""query {foo bar}""")
        assertZIO(response)(equalTo("{\"foo\":100,\"bar\":\"BAR\"}"))
      },
      test("two level") {
        val orc = Orc.make(
          Orc.node("Foo")(
            "value" -> Resolver.value("foo"),
            "bar"   -> Resolver.ref("Bar")
          ),
          Orc.node("Bar")(
            "foo"   -> Resolver.ref("Foo"),
            "value" -> Resolver.value("bar")
          )
        )

        val response = execute(orc)("{bar {foo {bar {foo {bar {value}}}}}}")
        assertZIO(response)(equalTo(
          "{\"bar\":{\"foo\":{\"bar\":{\"foo\":{\"bar\":{\"value\":\"bar\"}}}}}}"
        ))
      },
      test("list") {
        val orc = Orc.make(
          Orc.node("Query")("foos" -> Resolver.value(List("foo1", "foo2")))
        )

        val response = execute(orc)("{foos}")
        assertZIO(response)(equalTo("{\"foos\":[\"foo1\",\"foo2\"]}"))
      }
    )).provide(RemoteRuntime.live, EvaluationContext.live)
}
