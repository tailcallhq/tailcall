package com.tailcall.gateway.adt

import zio.json.{DeriveJsonCodec, JsonCodec}

final case class Endpoint(
  version: String,
  http: Option[Endpoint.HttpGlobal],
  baseURL: Option[String],
  endpoints: List[Endpoint.Item],
)

object Endpoint {
  final case class Item(name: String, http: Http)
  final case class Http(path: Route, method: Method = Method.GET)
  final case class HttpGlobal(baseURL: String)

  implicit val httpCodec: JsonCodec[Http]             = DeriveJsonCodec.gen[Http]
  implicit val globalHttpCodec: JsonCodec[HttpGlobal] = DeriveJsonCodec.gen[HttpGlobal]
  implicit val itemCodec: JsonCodec[Item]             = DeriveJsonCodec.gen[Item]
  implicit val endpointCodec: JsonCodec[Endpoint]     = DeriveJsonCodec.gen[Endpoint]
}
