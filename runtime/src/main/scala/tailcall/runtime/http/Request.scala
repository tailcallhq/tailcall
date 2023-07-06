package tailcall.runtime.http

import zio.Chunk
import zio.http.model.Headers
import zio.http.{Body, Request => ZRequest, URL}

import java.net.URI
final case class Request(
  url: String = "",
  method: Method = Method.GET,
  headers: Map[String, String] = Map.empty,
  body: Chunk[Byte] = Chunk.empty,
) {
  def toZHttpRequest: ZRequest =
    ZRequest(
      method = method.toZMethod,
      url = URL.fromString(url).getOrElse(throw new IllegalArgumentException(s"Invalid URL: $url")),
      headers = Headers(headers.map(header => Headers.Header(header._1, header._2))),
      version = zio.http.model.Version.`HTTP/1.1`,
      remoteAddress = None,
      body = Body.fromChunk(body),
    )

  def withBody(body: Chunk[Byte]): Request = copy(body = body)

  def unsafeRedirect(nurl: String): Request = {
    try {
      val originalURI  = new URI(url)
      val redirectPath = if (nurl.startsWith("/")) originalURI.resolve(nurl).toString else nurl
      copy(url = redirectPath)
    } catch { case _: Exception => throw new IllegalArgumentException(s"Invalid path: $nurl") }
  }
}
