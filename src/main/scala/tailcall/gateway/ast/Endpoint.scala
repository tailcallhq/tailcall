package tailcall.gateway.ast

import tailcall.gateway.ast.Path.Segment
import tailcall.gateway.http.{Method, Request}
import zio.Chunk
import zio.schema.meta.MetaSchema
import zio.schema.{DynamicValue, Schema}

final case class Endpoint(
  method: Method = Method.GET,
  path: Path = Path.empty,
  query: Chunk[(String, String)] = Chunk.empty,
  address: Endpoint.InetAddress,
  input: MetaSchema = Schema[Unit].ast,
  output: MetaSchema = Schema[Unit].ast,
  headers: Chunk[(String, String)] = Chunk.empty,
  protocol: Endpoint.Protocol = Endpoint.Protocol.Http,
  body: Option[String] = None
) {
  self =>
  def withMethod(method: Method): Endpoint = copy(method = method)

  def withPath(path: Path): Endpoint = copy(path = path)

  def withPath(path: String): Endpoint = copy(path = Path.unsafe.fromString(path))

  def withQuery(query: (String, String)*): Endpoint = copy(query = Chunk.from(query))

  def withAddress(address: Endpoint.InetAddress): Endpoint = copy(address = address)

  def withAddress(address: String): Endpoint = copy(address = Endpoint.inet(address))

  def withInput[A](implicit schema: Schema[A]): Endpoint = copy(input = schema.ast)

  def withOutput[A](implicit schema: Schema[A]): Endpoint = copy(output = schema.ast)

  def withProtocol(protocol: Endpoint.Protocol): Endpoint = copy(protocol = protocol)

  def withHttp: Endpoint = withProtocol(Endpoint.Protocol.Http)

  def withHttps: Endpoint = withProtocol(Endpoint.Protocol.Https)

  def withPort(port: Int): Endpoint = copy(address = address.copy(port = port))

  def withHeader(headers: (String, String)*): Endpoint = copy(headers = Chunk.from(headers))

  def withBody(body: String): Endpoint = copy(body = Option(body))

  def outputSchema: Schema[_] = output.toSchema

  def inputSchema: Schema[_] = input.toSchema

  def evaluate(input: DynamicValue): Request = Endpoint.evaluate(self, input)
}

object Endpoint {
  sealed trait Protocol {
    self =>
    def name: String =
      self match {
        case Protocol.Http  => "http"
        case Protocol.Https => "https"
      }
  }
  object Protocol       {
    case object Http  extends Protocol
    case object Https extends Protocol
  }

  sealed trait HttpError

  final case class InetAddress(host: String, port: Int = 80)

  def inet(host: String, port: Int = 80): InetAddress = InetAddress(host, port)

  def from(url: String): Endpoint = {
    val uri     = new java.net.URI(url)
    val path    = Path.unsafe.fromString(uri.getPath())
    val query   = Option(uri.getQuery).fold(Chunk.empty[(String, String)]) { query =>
      Chunk.from(query.split("&").map(_.split("=")).map { case Array(k, v) => k -> v })
    }
    val address = InetAddress(uri.getHost, uri.getPort)
    Endpoint(path = path, query = query, address = address)
  }

  def make(address: String): Endpoint = Endpoint(address = Endpoint.inet(address))

  def evaluate(endpoint: Endpoint, input: DynamicValue): Request = {
    val method     = endpoint.method
    val portString = endpoint.address.port match {
      case 80   => ""
      case 443  => ""
      case port => s":$port"
    }

    val queryString = endpoint.query.nonEmptyOrElse("")(_.map { case (k, v) => s"$k=${Mustache.evaluate(v, input)}" }
      .mkString("?", "&", ""))

    val pathString: String = endpoint.path.transform {
      case Segment.Literal(value)  => Path.Segment.Literal(value)
      case Segment.Param(mustache) => Path.Segment
          .Literal(mustache.evaluate(input).getOrElse(throw new RuntimeException("Mustache evaluation failed")))
    }.encode.getOrElse(throw new RuntimeException("Path encoding failed"))

    val url = List(endpoint.protocol.name, "://", endpoint.address.host, portString, pathString, queryString).mkString

    val headers = endpoint.headers.map { case (k, v) => k -> Mustache.evaluate(v, input) }.toMap

    Request(method = method, url = url, headers = headers)
  }
}
