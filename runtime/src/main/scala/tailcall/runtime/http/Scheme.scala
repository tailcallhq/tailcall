package tailcall.runtime.http

sealed trait Scheme {
  self =>
  def name: String =
    self match {
      case Scheme.Http  => "http"
      case Scheme.Https => "https"
    }
}

object Scheme {
  case object Http  extends Scheme
  case object Https extends Scheme
}
