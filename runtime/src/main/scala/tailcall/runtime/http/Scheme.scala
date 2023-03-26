package tailcall.runtime.http

import zio.json.{JsonCodec, jsonHint}

sealed trait Scheme {
  self =>
  def name: String =
    self match {
      case Scheme.Http  => "http"
      case Scheme.Https => "https"
    }
}

object Scheme {
  def fromString(input: String): Either[String, Scheme] =
    input.toLowerCase match {
      case "http"  => Right(Http)
      case "https" => Right(Https)
      case other   => Left(s"Unknown scheme: $other")
    }

  implicit val jsonCodec: JsonCodec[Scheme] = JsonCodec[String].transformOrFail[Scheme](fromString, _.name)

  @jsonHint("http")
  case object Http extends Scheme

  @jsonHint("https")
  case object Https extends Scheme
}
