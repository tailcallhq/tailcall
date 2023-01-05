package tailcall.gateway.adt

import caliban.parsing.adt.Document
import zio.json.{DeriveJsonCodec, JsonCodec}

final case class Config(version: String, endpoints: Endpoints, graphQL: GraphQL)

object Config {
  Document
  implicit val jsonCodec: JsonCodec[Config] = DeriveJsonCodec.gen[Config]
}
