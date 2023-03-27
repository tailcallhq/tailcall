package tailcall.runtime.ast

import tailcall.runtime.ast.Path.Segment
import tailcall.runtime.http.{Method, Request, Scheme}
import zio.Chunk
import zio.schema.{DynamicValue, Schema}

final case class Endpoint(
  method: Method = Method.GET,
  path: Path = Path.empty,
  query: Chunk[(String, String)] = Chunk.empty,
  address: Endpoint.InetAddress,
  input: Option[TSchema] = None,
  output: Option[TSchema] = None,
  headers: Chunk[(String, String)] = Chunk.empty,
  scheme: Scheme = Scheme.Http,
  body: Option[String] = None,
) {
  self =>
  def withMethod(method: Method): Endpoint = copy(method = method)

  def withPath(path: Path): Endpoint = copy(path = path)

  def withPath(path: String): Endpoint = copy(path = Path.unsafe.fromString(path))

  def withQuery(query: (String, String)*): Endpoint = copy(query = Chunk.from(query))

  def withAddress(address: Endpoint.InetAddress): Endpoint = copy(address = address)

  def withAddress(address: String): Endpoint = copy(address = Endpoint.inet(address))

  def withInput(schema: Option[TSchema]): Endpoint = copy(input = schema)

  def withOutput(schema: Option[TSchema]): Endpoint = copy(output = schema)

  def withProtocol(protocol: Scheme): Endpoint = copy(scheme = protocol)

  def withHttp: Endpoint = withProtocol(Scheme.Http)

  def withHttps: Endpoint = withProtocol(Scheme.Https)

  def withPort(port: Int): Endpoint = {
    if (port < 0 || port > 65535) throw new IllegalArgumentException("Port must be between 0 and 65535")
    copy(address = address.copy(port = port))
  }

  def withHeader(headers: (String, String)*): Endpoint = copy(headers = Chunk.from(headers))

  def withBody(body: String): Endpoint = copy(body = Option(body))

  lazy val outputSchema: Schema[Any] = output.map(TSchema.toZIOSchema).getOrElse(Schema[Unit]).asInstanceOf[Schema[Any]]

  lazy val inputSchema: Schema[Any] = input.map(TSchema.toZIOSchema).getOrElse(Schema[Unit]).asInstanceOf[Schema[Any]]

  def evaluate(input: DynamicValue): Request = Endpoint.evaluate(self, input)

  // TODO: add unit tests
  def url: String = {
    val portString = address.port match {
      case 80   => ""
      case 443  => ""
      case port => s":$port"
    }

    val queryString        = query.nonEmptyOrElse("")(_.map { case (k, v) => s"$k=$v" }.mkString("?", "&", ""))
    val pathString: String = path.encode.getOrElse(throw new RuntimeException("Path encoding failed"))
    List(scheme.name, "://", address.host, portString, pathString, queryString).mkString
  }
}

object Endpoint {
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

    val url = List(endpoint.scheme.name, "://", endpoint.address.host, portString, pathString, queryString).mkString

    val headers = endpoint.headers.map { case (k, v) => k -> Mustache.evaluate(v, input) }.toMap

    Request(method = method, url = url, headers = headers)
  }
}
