package tailcall.gateway.http

import tailcall.gateway.ast.Path.Segment
import tailcall.gateway.ast.{Endpoint, Mustache, Path}
import zio.schema.DynamicValue

object EndpointCompiler {
  final case class Request(
    url: String = "",
    method: String = "GET",
    headers: Map[String, String] = Map.empty,
    body: Array[Byte] = Array.empty
  )

  def compile(endpoint: Endpoint, input: DynamicValue): Request = {

    val method = endpoint.method.name

    val portString = endpoint.address.port match {
      case 80   => ""
      case 443  => ""
      case port => s":$port"
    }

    val queryString = endpoint
      .query
      .nonEmptyOrElse("")(
        _.map { case (k, v) => s"$k=${Mustache.evaluate(v, input)}" }.mkString("?", "&", "")
      )

    val pathString: String = endpoint
      .path
      .transform {
        case Segment.Literal(value)  => Path.Segment.Literal(value)
        case Segment.Param(mustache) => Path
            .Segment
            .Literal(
              mustache
                .evaluate(input)
                .getOrElse(throw new RuntimeException("Mustache evaluation failed"))
            )
      }
      .encode
      .getOrElse(throw new RuntimeException("Path encoding failed"))

    val url = List(
      endpoint.protocol.name,
      "://",
      endpoint.address.host,
      portString,
      pathString,
      queryString
    ).mkString

    val headers = endpoint.headers.map { case (k, v) => k -> Mustache.evaluate(v, input) }.toMap
    Request(method = method, url = url, headers = headers)
  }
}
