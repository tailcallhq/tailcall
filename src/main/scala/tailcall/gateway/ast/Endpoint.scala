package tailcall.gateway.ast

sealed trait Endpoint

object Endpoint {
  final case class InetAddress(host: String, port: Int = 80)

  final case class Http(
    method: Method = Method.GET,
    path: Path,
    address: InetAddress,
    input: Http.Input = Http.Input(None, None, None),
    output: Http.Output = Http.Output(None)
  ) extends Endpoint

  object Http {
    final case class Input(query: Option[TSchema], path: Option[TSchema], body: Option[TSchema])
    final case class Output(body: Option[TSchema])
  }
}
