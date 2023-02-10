package tailcall.gateway.ast

final case class Endpoint(
  method: Method = Method.GET,
  path: Path = Path.empty,
  query: Map[String, String] = Map.empty,
  address: Endpoint.InetAddress,
  input: TSchema = TSchema.unit,
  output: TSchema = TSchema.unit
) {
  def withMethod(method: Method): Endpoint = copy(method = method)

  def withPath(path: Path): Endpoint = copy(path = path)

  def withPath(path: String): Endpoint = copy(path = Path.unsafe.fromString(path))

  def withQuery(query: Map[String, String]): Endpoint = copy(query = query)

  def withAddress(address: Endpoint.InetAddress): Endpoint = copy(address = address)

  def withAddress(address: String): Endpoint = copy(address = Endpoint.inet(address))

  def withInput(input: TSchema): Endpoint = copy(input = input)

  def withOutput(output: TSchema): Endpoint = copy(output = output)
}

object Endpoint {
  sealed trait HttpError

  final case class InetAddress(host: String, port: Int = 80)

  def inet(host: String, port: Int = 80): InetAddress = InetAddress(host, port)

  def from(url: String): Endpoint = {
    val uri     = new java.net.URI(url)
    val path    = Path.unsafe.fromString(uri.getPath())
    val query   = Option(uri.getQuery).fold(Map.empty[String, String]) { query =>
      query.split("&").map(_.split("=")).map { case Array(k, v) => k -> v }.toMap
    }
    val address = InetAddress(uri.getHost, uri.getPort)
    Endpoint(path = path, query = query, address = address)
  }

  def make(address: String): Endpoint = Endpoint(address = Endpoint.inet(address))
}
