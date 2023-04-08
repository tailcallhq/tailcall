package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.json.{DeriveJsonCodec, JsonCodec, jsonHint}

import java.net.URL

@jsonHint("server")
final case class Server(baseURL: Option[URL] = None) {
  self =>
  def isEmpty: Boolean                  = baseURL.isEmpty
  def mergeRight(other: Server): Server = Server(baseURL = other.baseURL.orElse(self.baseURL))
}

object Server {
  implicit val urlCodec: JsonCodec[URL]          = JsonCodec[String].transformOrFail[URL](
    string =>
      try Right(new URL(string))
      catch { case _: Throwable => Left(s"Malformed url: ${string}") },
    _.toString,
  )
  implicit val json: JsonCodec[Server]           = DeriveJsonCodec.gen[Server]
  implicit val directive: DirectiveCodec[Server] = DirectiveCodec.fromJsonCodec("server", json)
}
