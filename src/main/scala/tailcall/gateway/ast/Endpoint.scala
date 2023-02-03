package tailcall.gateway.ast

sealed trait Endpoint

object Endpoint {
  case class Http(method: Method, path: Path) extends Endpoint
}
