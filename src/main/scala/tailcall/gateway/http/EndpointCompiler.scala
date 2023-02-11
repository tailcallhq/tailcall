package tailcall.gateway.http

import tailcall.gateway.ast.Endpoint
import zio.schema.DynamicValue
import tailcall.gateway.ast.Path
import scala.annotation.unused
import tailcall.gateway.internal.DynamicValueExtension._

object EndpointCompiler {
  final case class Request(
    url: String = "",
    method: String = "GET",
    headers: Map[String, String] = Map.empty,
    body: Array[Byte] = Array.empty
  )

  def compile(
    endpoint: Endpoint,
    @unused
    input: DynamicValue
  ): Request = {

    val method = endpoint.method.name

    val portString = endpoint.address.port match {
      case 80   => ""
      case 443  => ""
      case port => s":$port"
    }

    val queryString = endpoint
      .query
      .nonEmptyOrElse("")(_.map { case (k, v) => s"$k=$v" }.mkString("?", "&", ""))

    val pathString = endpoint
      .path
      .transform { segment =>
        segment match {
          case Path.Segment.Literal(value)     => Path.Segment.Literal(value)
          case Path.Segment.Param(placeholder) =>
            placeholder.evaluate(input).flatMap(_.asPrimitive) match {
              case None        => throw new RuntimeException("Missing placeholder value")
              case Some(value) => Path.Segment.Literal(value.value.toString)
            }
        }
      }
      .encode
      .getOrElse(throw new RuntimeException("Invalid path"))

    val url = List(
      endpoint.protocol.name,
      "://",
      endpoint.address.host,
      portString,
      pathString,
      queryString
    ).mkString
    Request(method = method, url = url)
  }
}
