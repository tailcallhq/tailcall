package tailcall.runtime

import tailcall.runtime.model.{Endpoint, Method}
import tailcall.test.TailcallSpec
import zio.schema.DynamicValue
import zio.test._

import java.nio.charset.StandardCharsets

object EndpointSpec extends TailcallSpec {
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
          "http://abc.com/abc?a=b"  -> root.withQuery("a" -> "b").withPath("/abc"),
          "http://abc.com/abc?a=b"  -> root.withPath("/abc").withQuery("a" -> "b"),
          "http://abc.com:8080"     -> root.withPort(8080),
          "http://abc.com:8080/abc" -> root.withPort(8080).withPath("/abc"),
          "http://abc.com/abc"      -> root.withPath("/abc").withPort(80),
          "http://abc.com/abc"      -> root.withPath("/abc").withPort(443),
        )

        checkAll(Gen.fromIterable(inputs)) { case (expected, endpoint) =>
          val request = endpoint.evaluate(DynamicValue(()))
          assertTrue(request.url == expected)
        }
      },
      test("{{path}}") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          DynamicValue(Map("a" -> 1))             -> root.withPath("/users/{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> 1))) -> root.withPath("/users/{{a.b}}"),
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val request = endpoint.evaluate(input)
          assertTrue(request.url == "http://abc.com/users/1")
        }
      },
      test("headers") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          DynamicValue(Map("a" -> "Tailcall"))             -> root.withHeader("X-Server" -> "{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> "Tailcall"))) -> root.withHeader("X-Server" -> "{{a.b}}"),
          DynamicValue(Map("a" -> "Tailcall")) -> root.withHeader("X-Server" -> "{{a}}", "X-Ignore" -> "{{b}}"),
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val actual   = endpoint.evaluate(input).headers
          val expected = Map("X-Server" -> "Tailcall", "content-length" -> "0", "content-type" -> "application/json")
          assertTrue(actual == expected)
        }
      },
      test("query params") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          DynamicValue(())                          -> root.withQuery("a" -> "1"),
          DynamicValue(Map("a" -> "1"))             -> root.withQuery("a" -> "{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> "1"))) -> root.withQuery("a" -> "{{a.b}}"),
          DynamicValue(Map("a" -> "1"))             -> root.withQuery("a" -> "{{a}}", "b" -> "{{b}}"),
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val request = endpoint.evaluate(input)
          assertTrue(request.url == "http://abc.com?a=1")
        }
      },
      test("query with list params") {
        val root   = Endpoint.make("abc.com")
        val inputs = List(
          DynamicValue(Map("a" -> List("1", "2", "3")))                         -> root.withQuery("a" -> "{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> List("1", "2", "3"))))             -> root.withQuery("a" -> "{{a.b}}"),
          DynamicValue(Map("a" -> Map("b" -> Map("c" -> List("1", "2", "3"))))) -> root.withQuery("a" -> "{{a.b.c}}"),
          DynamicValue(List(Map("a" -> "1"), Map("a" -> "2"), Map("a" -> "3"))) -> root.withQuery("a" -> "{{a}}"),
          DynamicValue(
            List(Map("a" -> Map("b" -> "1")), Map("a" -> Map("b" -> "2")), Map("a" -> Map("b" -> "3")))
          )                                                                     -> root.withQuery("a" -> "{{a.b}}"),
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val request = endpoint.evaluate(input)
          assertTrue(request.url == "http://abc.com?a=1&a=2&a=3")
        }
      },
      test("query with duplicate params") {
        val input    = DynamicValue(Map("a" -> List("1", "1", "2")))
        val endpoint = Endpoint.make("abc.com").withQuery("a" -> "{{a}}")
        val request  = endpoint.evaluate(input)
        assertTrue(request.url == "http://abc.com?a=1&a=2")
      },
      test("body") {
        val endpoint = Endpoint.post("abc.com")
        val inputs   = List(
          DynamicValue(Map("a" -> "1"))             -> endpoint.withBody("{{a}}"),
          DynamicValue(Map("a" -> Map("b" -> "1"))) -> endpoint.withBody("{{a.b}}"),
        )

        checkAll(Gen.fromIterable(inputs)) { case (input, endpoint) =>
          val request = endpoint.evaluate(input)
          val body    = new String(request.body.toArray, StandardCharsets.UTF_8)
          assertTrue(body == """"1"""")
        }
      },
      test("noBody") {
        val request = Endpoint.post("abc.com").evaluate(DynamicValue(Map("a" -> "1")))
        val body    = new String(request.body.toArray, StandardCharsets.UTF_8)
        assertTrue(body == """{"a":"1"}""")
      },
    )
}
