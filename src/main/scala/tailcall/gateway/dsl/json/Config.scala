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

  final case class Field(
    @jsonField("type") as: String,
    isList: Option[Boolean] = None,
    isRequired: Option[Boolean] = None,
    steps: Option[List[Step]] = None,
    args: Option[Map[String, TSchema]] = None
  ) {
    def withList: Field                                  = copy(isList = Option(true))
    def withRequired: Field                              = copy(isRequired = Option(true))
    def withArguments(args: Map[String, TSchema]): Field = copy(args = Option(args))
    def apply(args: (String, TSchema)*): Field           = copy(args = Option(args.toMap))
  }

  object Field {
    def apply(str: String, operations: Step*): Field =
      Field(as = str, steps = if (operations.isEmpty) None else Option(operations.toList))
  }

  sealed trait Step
  object Step {
    @jsonHint("$http")
    final case class Http(
      path: Path,
      method: Option[Method] = None,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None
    ) extends Step
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

  implicit val operationCodec: JsonCodec[Step]                    = DeriveJsonCodec.gen[Step]
  implicit val fieldDefinitionCodec: JsonCodec[Field]             = DeriveJsonCodec.gen[Field]
  implicit val schemaDefinitionCodec: JsonCodec[SchemaDefinition] = DeriveJsonCodec.gen[SchemaDefinition]
  implicit val graphQLCodec: JsonCodec[GraphQL]                   = DeriveJsonCodec.gen[GraphQL]
  implicit val serverCodec: JsonCodec[Server]                     = DeriveJsonCodec.gen[Server]
  implicit val jsonCodec: JsonCodec[Config]                       = DeriveJsonCodec.gen[Config]
}
