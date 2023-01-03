package com.tailcall.gateway.adt

import zio.json.JsonCodec
import caliban.parsing.adt.Document
import zio.json.DeriveJsonCodec
import zio.json.DeriveJsonDecoder
import zio.schema.DeriveSchema

final case class Config(endpoints: List[Endpoint], graphQL: GraphQL)

object Config {
  Document
  implicit val jsonCodec: JsonCodec[Config] = DeriveJsonCodec.gen[Config]
}
