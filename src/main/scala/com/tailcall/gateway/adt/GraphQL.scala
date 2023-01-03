package com.tailcall.gateway.adt

import zio.json.JsonCodec
import zio.json.DeriveJsonCodec

final case class GraphQL()
object GraphQL {
  implicit val jsonCodec: JsonCodec[GraphQL] = DeriveJsonCodec.gen[GraphQL]
}
