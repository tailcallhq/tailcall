package tailcall.gateway.ast

sealed trait Endpoint

object Endpoint {
  sealed trait HttpError

  final case class InetAddress(host: String, port: Int = 80)

  final case class Http(
    method: Method = Method.GET,
    path: Path,
    query: Map[String, String] = Map.empty,
    address: InetAddress,
    input: TSchema,
    output: TSchema
  ) extends Endpoint

  def http(
    method: Method = Method.GET,
    path: Path,
    query: Map[String, String] = Map.empty,
    address: InetAddress,
    input: TSchema = TSchema.unit,
    output: TSchema = TSchema.unit
  ): Http = Http(method, path, query, address, input, output)

  def inet(host: String, port: Int = 80): InetAddress = InetAddress(host, port)
}
