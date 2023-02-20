package tailcall.gateway.http

import io.netty.handler.codec.http._

final case class Request(
  url: String = "",
  method: Method = Method.GET,
  headers: Map[String, String] = Map.empty,
  body: Array[Byte] = Array.empty
) {
  def toHttpRequest: FullHttpRequest = {
    val httpMethod  = HttpMethod.valueOf(method.name)
    val httpRequest = new DefaultFullHttpRequest(HttpVersion.HTTP_1_1, httpMethod, url)

    headers.foreach { case (key, value) => httpRequest.headers.add(key, value) }
    if (body.nonEmpty) { httpRequest.content().writeBytes(body) }

    httpRequest
  }
}
