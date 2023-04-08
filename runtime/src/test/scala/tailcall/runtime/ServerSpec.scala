package tailcall.runtime

import caliban.Value.StringValue
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec.{DecoderSyntax, EncoderSyntax}
import tailcall.runtime.model.Server
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

import java.net.URL

object ServerSpec extends ZIOSpecDefault {
  def spec =
    suite("ServerSpec")(suite("directive")(
      test("encoding") {
        val server   = Server(baseURL = Some(new URL("http://localhost:8080")))
        val actual   = server.toDirective
        val expected = Directive(name = "server", arguments = Map("baseURL" -> StringValue("http://localhost:8080")))
        assertZIO(actual.toZIO)(equalTo(expected))
      },
      test("decode") {
        val directive = Directive(name = "server", arguments = Map("baseURL" -> StringValue("http://localhost:8080")))
        val actual    = directive.fromDirective[Server]
        val expected  = Server(baseURL = Some(new URL("http://localhost:8080")))
        assertZIO(actual.toZIO)(equalTo(expected))
      },
    ))
}
