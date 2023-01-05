package tailcall.gateway.adt

import zio.json.{DeriveJsonCodec, JsonCodec}

final case class Endpoints(
  http: Option[Endpoints.HttpGlobal],
  baseURL: Option[String],
  definitions: List[Endpoints.Definition],
)

object Endpoints {
  final case class Definition(name: String, http: Http)
  final case class Http(path: Route, method: Method = Method.GET)
  final case class HttpGlobal(baseURL: String)

  implicit val httpCodec: JsonCodec[Http]             = DeriveJsonCodec.gen[Http]
  implicit val globalHttpCodec: JsonCodec[HttpGlobal] = DeriveJsonCodec.gen[HttpGlobal]
  implicit val itemCodec: JsonCodec[Definition]       = DeriveJsonCodec.gen[Definition]
  implicit val endpointCodec: JsonCodec[Endpoints]    = DeriveJsonCodec.gen[Endpoints]
}
