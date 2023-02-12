package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
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
  def withMethod(method: Method): Endpoint = copy(method = method)

  def withPath(path: Path): Endpoint = copy(path = path)

  def withPath(path: String): Endpoint = copy(path = Path.unsafe.fromString(path))

  def withQuery(query: (String, String)*): Endpoint = copy(query = Chunk.from(query))

  def withAddress(address: Endpoint.InetAddress): Endpoint = copy(address = address)

  def withAddress(address: String): Endpoint = copy(address = Endpoint.inet(address))

  def withInput[A](implicit schema: Schema[A]): Endpoint = copy(input = schema.ast)

  def withOutput[A](implicit schema: Schema[A]): Endpoint = copy(output = schema.ast)

  def remote: Remote[DynamicValue => DynamicValue] = Remote.fromEndpoint(this)

  def apply(input: Remote[DynamicValue]): Remote[DynamicValue] = remote(input)

  def withProtocol(protocol: Endpoint.Protocol): Endpoint = copy(protocol = protocol)

  def withHttp: Endpoint = withProtocol(Endpoint.Protocol.Http)

  def withHttps: Endpoint = withProtocol(Endpoint.Protocol.Https)

  def withPort(port: Int): Endpoint = copy(address = address.copy(port = port))

  def withHeader(headers: (String, String)*): Endpoint = copy(headers = Chunk.from(headers))

  def withBody(body: String): Endpoint = copy(body = Option(body))
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
}
