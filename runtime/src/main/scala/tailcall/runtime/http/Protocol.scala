package tailcall.runtime.http

sealed trait Protocol {
  self =>
  def name: String =
    self match {
      case Protocol.Http  => "http"
      case Protocol.Https => "https"
    }
}

object Protocol {
  case object Http  extends Protocol
  case object Https extends Protocol
}
