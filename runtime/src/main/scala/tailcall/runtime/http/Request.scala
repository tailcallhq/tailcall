package tailcall.runtime.http

import zio.Chunk
import zio.http.{Body, Header, Headers, URL, Request as ZRequest}
final case class Request(
  url: String = "",
  method: Method = Method.GET,
  headers: Map[String, String] = Map.empty,
  body: Chunk[Byte] = Chunk.empty,
) {
  def toZHttpRequest: ZRequest =
    ZRequest(
      method = method.toZMethod,
      url = URL.decode(url).getOrElse(throw new IllegalArgumentException(s"Invalid URL: $url")),
      headers = Headers(headers.map(header => Header.Custom(header._1, header._2))),
      version = zio.http.Version.`HTTP/1.1`,
      remoteAddress = None,
      body = Body.fromChunk(body),
    )

  def withBody(body: Chunk[Byte]): Request = copy(body = body)
}
