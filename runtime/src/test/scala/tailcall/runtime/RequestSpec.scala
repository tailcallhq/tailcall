package tailcall.runtime

import tailcall.runtime.model.Request
import tailcall.test.TailcallSpec
import zio.Scope
import zio.test.{Spec, TestEnvironment, assertTrue}

object RequestSpec extends TailcallSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("RequestSpec")(
      test("request redirect") {
        val request  = Request(url = "http://abc.com/foo")
        val actual   = request.unsafeRedirect("/bar").url
        val expected = "http://abc.com/bar"

        assertTrue(actual == expected)
      },
      test("full url redirect") {
        val request  = Request(url = "http://abc.com/foo")
        val actual   = request.unsafeRedirect("https://abc.com/foo").url
        val expected = "https://abc.com/foo"

        assertTrue(actual == expected)
      },
    )
}
