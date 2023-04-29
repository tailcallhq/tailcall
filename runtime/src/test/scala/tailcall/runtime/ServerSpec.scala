package tailcall.runtime

import caliban.InputValue.ObjectValue
import caliban.Value.IntValue.IntNumber
import caliban.Value.StringValue
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec.{DecoderSyntax, EncoderSyntax}
import tailcall.runtime.model.Server
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}
import zio.test.TestResultZIOOps
import java.net.URL

object ServerSpec extends ZIOSpecDefault {
  def spec =
    suite("ServerSpec")(suite("directive")(
      test("baseURL") {
        val directive = Directive(name = "server", arguments = Map("baseURL" -> StringValue("http://localhost:8080")))
        val actual    = directive.fromDirective[Server]
        val expected  = Server(baseURL = Some(new URL("http://localhost:8080")))
        assertZIO(actual.toZIO)(equalTo(expected)) && assertZIO(expected.toDirective.toZIO)(equalTo(directive))
      },
      test("timeout") {
        val directive = Directive(
          name = "server",
          arguments = Map("baseURL" -> StringValue("http://localhost:8080"), "timeout" -> IntNumber(1000)),
        )
        val actual    = directive.fromDirective[Server]
        val expected  = Server(baseURL = Some(new URL("http://localhost:8080")), timeout = Some(1000))
        assertZIO(actual.toZIO)(equalTo(expected)) && assertZIO(expected.toDirective.toZIO)(equalTo(directive))
      },
      test("vars") {
        val directive = Directive(
          name = "server",
          arguments = Map(
            "baseURL" -> StringValue("http://localhost:8080"),
            "timeout" -> IntNumber(1000),
            "vars"    -> ObjectValue(Map("foo" -> StringValue("bar"))),
          ),
        )
        val actual    = directive.fromDirective[Server]
        val expected  = Server(
          baseURL = Some(new URL("http://localhost:8080")),
          timeout = Some(1000),
          vars = Option(Map("foo" -> "bar")),
        )
        assertZIO(actual.toZIO)(equalTo(expected)) &&
        assertZIO(expected.toDirective.toZIO)(equalTo(directive))
      },
    ))
}
