package tailcall.gateway

import tailcall.gateway.ast.Endpoint
import tailcall.gateway.http.Method
import zio.schema.DynamicValue
import zio.test._

object EndpointSpec extends ZIOSpecDefault {
  def spec =
    suite("EndpointSpec")(
      test("method") {
        val endpoint = Endpoint.make("abc.com").withMethod(Method.POST)
        val request  = endpoint.evaluate(DynamicValue(()))
        assertTrue(request.method == Method.POST)
      },
      test("path") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          "http://abc.com/abc"      -> root.withPath("/abc"),
          "https://abc.com/abc"     -> root.withPath("/abc").withHttps,
          "http://abc.com/abc?a=b"  -> root
            .withQuery("a" -> "b")
            .withPath("/abc"),
          "http://abc.com/abc?a=b"  -> root
            .withPath("/abc")
            .withQuery("a" -> "b"),
          "http://abc.com:8080"     -> root.withPort(8080),
          "http://abc.com:8080/abc" -> root.withPort(8080).withPath("/abc"),
          "http://abc.com/abc"      -> root.withPath("/abc").withPort(80),
          "http://abc.com/abc"      -> root.withPath("/abc").withPort(443)
        )

        checkAll(Gen.fromIterable(inputs)) { case (expected, endpoint) =>
          val request = endpoint.evaluate(DynamicValue(()))
          assertTrue(request.url == expected)
        }
      },
      test("{{path}}") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          DynamicValue(Map("a" -> 1)) -> root.withPath("/users/{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> 1))) -> root
            .withPath("/users/{{a.b}}")
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val request = endpoint.evaluate(input)
          assertTrue(request.url == "http://abc.com/users/1")
        }
      },
      test("headers") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          DynamicValue(Map("a" -> "1"))             -> root
            .withHeader("X-Server" -> "{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> "1"))) -> root
            .withHeader("X-Server" -> "{{a.b}}")
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val request = endpoint.evaluate(input)
          assertTrue(request.headers == Map("X-Server" -> "1"))
        }
      },
      test("query") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          DynamicValue(())              -> root.withQuery("a" -> "1"),
          DynamicValue(Map("a" -> "1")) -> root.withQuery("a" -> "{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> "1"))) -> root
            .withQuery("a" -> "{{a.b}}")
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val request = endpoint.evaluate(input)
          assertTrue(request.url == "http://abc.com?a=1")
        }
      }
    )
}
