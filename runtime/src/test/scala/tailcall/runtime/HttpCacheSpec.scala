package tailcall.runtime

import tailcall.runtime.service.HttpCache
import tailcall.runtime.service.HttpCache.dateFormat
import tailcall.test.TailcallSpec
import zio._
import zio.http.Response
import zio.http.model.Headers
import zio.http.model.Headers.Header
import zio.test.Assertion._
import zio.test._

import java.time.Instant

object HttpCacheSpec extends TailcallSpec {
  def spec = {
    suite("HttpCacheSpec Cache-Control")(
      test("ttl") {
        val ttl      = HttpCache.ttl(Response.ok.addHeaders(headers = Header("Cache-Control", "max-age=1000")))
        val expected = Some(Duration.fromSeconds(1000))
        assert(ttl)(equalTo(expected))
      },
      test("ttl cache-control") {
        val ttl      = HttpCache.ttl(Response.ok.addHeaders(headers = Header("cache-control", "max-age=1000")))
        val expected = Some(Duration.fromSeconds(1000))
        assert(ttl)(equalTo(expected))
      },
      test("ttl cache-control private") {
        val ttl      = HttpCache.ttl(Response.ok.addHeaders(headers = Header("cache-control", "max-age=1000, private")))
        val expected = None
        assert(ttl)(equalTo(expected))
      },
      test("expires -1") {
        val ttl      = HttpCache.ttl(Response.ok.addHeaders(headers = Header("expires", "-1")))
        val expected = None
        assert(ttl)(equalTo(expected))
      },
      test("cache-control and expires") {
        lazy val expiry = Instant.now().plusSeconds(1000).toString
        val ttl         = HttpCache.ttl(
          Response.ok.addHeaders(headers = Headers(Header("expires", expiry), Header("cache-control", "max-age=2000")))
        )
        val expected    = Some(Duration.fromSeconds(2000))
        assert(ttl)(equalTo(expected))
      },
      test("expires after 1000 second") {
        val now        = Instant.parse("2021-01-01T00:00:00Z")
        val headerTime = dateFormat.format(now.toEpochMilli + 1000000L)
        val p          = HttpCache.ttl(Response.ok.addHeaders(headers = Header("expires", headerTime)), now)
        val expected   = Some(Duration.fromSeconds(1000))
        assert(p)(equalTo(expected))
      },
      suite("calculate Minimum Headers")(
        test("private") {
          val headers  = List(
            ("Cache-Control", "public, max-age=3600"),
            ("Cache-Control", "private, max-age=1800"),
            ("Expires", "Fri, 17 Jun 2023 12:00:00 GMT"),
            ("Expires", "Fri, 17 Jun 2023 13:30:00 GMT"),
          )
          val expected = Some("Cache-Control: private")
          val actual   = HttpCache.calculateMinimumHeader(headers)
          assertTrue(actual == expected)
        },
        test("default values") {
          val headers = List(("Cache-Control", "max-age=0"), ("Expires", "-1"))
          assertTrue(HttpCache.calculateMinimumHeader(headers).isEmpty)
        },
        test("Minimum possible input") {
          val headers = List(("Cache-Control", "max-age=0"), ("Expires", "Thu, 01 Jan 1970 00:00:00 GMT"))
          assertTrue(HttpCache.calculateMinimumHeader(headers).isEmpty)
        },
        test("Multiple Cache-Control headers with different max-age values:") {
          val headers = List(
            ("Cache-Control", "max-age=3600"),
            ("Cache-Control", "max-age=1800"),
            ("Expires", "Fri, 17 Jun 2023 12:00:00 GMT"),
            ("Expires", "Fri, 17 Jun 2023 13:30:00 GMT"),
          )
          assertTrue(HttpCache.calculateMinimumHeader(headers).contains("Cache-Control: max-age=1800"))

        },
      ),
    )
  }

}
