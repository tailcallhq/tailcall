package tailcall.gateway.dsl.json

import tailcall.gateway.ast._
import tailcall.gateway.dsl.json.Config._
import tailcall.gateway.http.Method
import zio.json._

final case class Config(version: Int = 0, server: Server, graphQL: GraphQL = GraphQL())

object Config {
  final case class Server(baseURL: String)
  final case class SchemaDefinition(query: Option[String] = None, mutation: Option[String] = None)
  final case class GraphQL(
    schema: SchemaDefinition = SchemaDefinition(),
    types: Map[String, Map[String, Field]] = Map.empty
  )

  final case class Field(as: String, resolve: Option[List[Operation]] = None)
  object Field {
    def apply(str: String, operations: Operation*): Field =
      Field(str, if (operations.isEmpty) None else Option(operations.toList))
  }

  @jsonDiscriminator("type")
  sealed trait Operation
  object Operation {
    @jsonHint("http")
    final case class Http(
      path: Path,
      method: Method = Method.GET,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None
    ) extends Operation
  }

  /**
   * Json Codecs
   *
   * TODO: This should only be done once, not for every
   * instance of Config. This is done currently because if
   * we create a jsonCodec from Schema, internally Maps are
   * stored as chunks and the outputted json looks like a
   * list of tuples.
   */

  implicit val operationCodec: JsonCodec[Operation]               = DeriveJsonCodec.gen[Operation]
  implicit val fieldDefinitionCodec: JsonCodec[Field]             = DeriveJsonCodec.gen[Field]
  implicit val schemaDefinitionCodec: JsonCodec[SchemaDefinition] = DeriveJsonCodec.gen[SchemaDefinition]
  implicit val graphQLCodec: JsonCodec[GraphQL]                   = DeriveJsonCodec.gen[GraphQL]
  implicit val serverCodec: JsonCodec[Server]                     = DeriveJsonCodec.gen[Server]
  implicit val jsonCodec: JsonCodec[Config]                       = DeriveJsonCodec.gen[Config]
}
