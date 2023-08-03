package tailcall.runtime.model

import zio.http.model.{Method => ZMethod}
import zio.json.JsonCodec
sealed trait Method {
  def name: String       = Method.encode(this)
  def toZMethod: ZMethod = ZMethod.fromString(name)
}

object Method {
  case object GET     extends Method
  case object POST    extends Method
  case object PUT     extends Method
  case object DELETE  extends Method
  case object HEAD    extends Method
  case object OPTIONS extends Method
  case object PATCH   extends Method
  case object CONNECT extends Method
  case object TRACE   extends Method

  def encode(method: Method): String =
    method match {
      case Method.GET     => "GET"
      case Method.POST    => "POST"
      case Method.PUT     => "PUT"
      case Method.DELETE  => "DELETE"
      case Method.HEAD    => "HEAD"
      case Method.OPTIONS => "OPTIONS"
      case Method.PATCH   => "PATCH"
      case Method.CONNECT => "CONNECT"
      case Method.TRACE   => "TRACE"
    }

  def decode(method: String): Either[String, Method] =
    method match {
      case "GET"    => Right(Method.GET)
      case "POST"   => Right(Method.POST)
      case "PUT"    => Right(Method.PUT)
      case "DELETE" => Right(Method.DELETE)
      case name     => Left("Unknown method: " + name)
    }

  implicit lazy val methodCodec: JsonCodec[Method] = JsonCodec[String].transformOrFail(Method.decode, Method.encode)
}
