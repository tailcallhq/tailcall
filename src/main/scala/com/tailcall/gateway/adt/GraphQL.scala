package com.tailcall.gateway.adt

import zio.json.{DeriveJsonCodec, JsonCodec}

final case class GraphQL()
object GraphQL {
  implicit val jsonCodec: JsonCodec[GraphQL] = DeriveJsonCodec.gen[GraphQL]
}
