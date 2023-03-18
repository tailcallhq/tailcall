package tailcall.runtime.dsl.json

import tailcall.runtime.ast._
import tailcall.runtime.dsl.json.Config._
import tailcall.runtime.http.Method
import tailcall.runtime.service.ConfigFileReader
import tailcall.runtime.transcoder.Syntax
import zio.ZIO
import zio.json._
import zio.json.ast.Json

import java.io.File

final case class Config(version: Int = 0, server: Server, graphQL: GraphQL = GraphQL()) {
  self =>
  def toBlueprint: Blueprint =
    self.transcode[Blueprint].getOrElse(throw new RuntimeException("Failed to transcode config to blueprint"))

  def mergeRight(other: Config): Config = {
    Config(
      version = other.version,
      server = self.server.mergeRight(other.server),
      graphQL = self.graphQL.mergeRight(other.graphQL)
    )
  }
}

object Config {
  final case class Server(host: String, port: Option[Int] = None) {
    self =>
    def mergeRight(other: Server): Server = Server(host = other.host, port = other.port.orElse(self.port))
  }

  final case class SchemaDefinition(query: Option[String] = None, mutation: Option[String] = None)
  final case class GraphQL(
    schema: SchemaDefinition = SchemaDefinition(),
    types: Map[String, Map[String, Field]] = Map.empty
  ) {
    self =>
    def mergeRight(other: GraphQL): GraphQL =
      GraphQL(
        schema = SchemaDefinition(
          query = other.schema.query.orElse(self.schema.query),
          mutation = other.schema.mutation.orElse(self.schema.mutation)
        ),
        types = self.types ++ other.types
      )
  }

  // TODO: Field and Argument can be merged
  final case class Field(
    @jsonField("type") typeOf: String,
    isList: Option[Boolean] = None,
    isRequired: Option[Boolean] = None,
    steps: Option[List[Step]] = None,
    args: Option[Map[String, Argument]] = None
  ) {
    def asList: Field                                     = copy(isList = Option(true))
    def asRequired: Field                                 = copy(isRequired = Option(true))
    def withArguments(args: Map[String, Argument]): Field = copy(args = Option(args))
    def apply(args: (String, Argument)*): Field           = copy(args = Option(args.toMap))
  }

  object Field {
    def apply(str: String, operations: Step*): Field =
      Field(typeOf = str, steps = if (operations.isEmpty) None else Option(operations.toList))

    def string: Field = Field(typeOf = "String")
    def int: Field    = Field(typeOf = "Int")
    def bool: Field   = Field(typeOf = "Boolean")
  }

  sealed trait Step
  object Step {
    @jsonHint("$http")
    final case class Http(
      path: Path,
      method: Option[Method] = None,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None
    ) extends Step {
      def withOutput(output: TSchema): Http = copy(output = Option(output))
      def withInput(input: TSchema): Http   = copy(input = Option(input))
    }

    @jsonHint("$const")
    final case class Constant(json: Json) extends Step
    object Constant {
      implicit val codec: JsonCodec[Constant] = JsonCodec(Json.encoder, Json.decoder).transform(Constant(_), _.json)
    }

    @jsonHint("$objectPath")
    final case class ObjPath(map: Map[String, List[String]]) extends Step
    object ObjPath {
      def apply(map: (String, List[String])*): ObjPath = ObjPath(map.toMap)
      implicit val codec: JsonCodec[ObjPath] = JsonCodec[Map[String, List[String]]].transform(ObjPath(_), _.map)
    }
  }

  final case class Argument(
    @jsonField("type") typeOf: String,
    isList: Option[Boolean] = None,
    isRequired: Option[Boolean] = None
  ) {
    self =>
    def asList: Argument     = self.copy(isList = Option(true))
    def asRequired: Argument = self.copy(isRequired = Option(true))
  }

  object Argument {
    val string: Argument = Argument("String")
    val int: Argument    = Argument("Int")
    val bool: Argument   = Argument("Boolean")
  }

  def fromFile(file: File): ZIO[ConfigFileReader, Throwable, Config] = ConfigFileReader.readFile(file)

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
  implicit val inputTypeCodec: JsonCodec[Argument]                = DeriveJsonCodec.gen[Argument]
  implicit val fieldDefinitionCodec: JsonCodec[Field]             = DeriveJsonCodec.gen[Field]
  implicit val schemaDefinitionCodec: JsonCodec[SchemaDefinition] = DeriveJsonCodec.gen[SchemaDefinition]
  implicit val graphQLCodec: JsonCodec[GraphQL]                   = DeriveJsonCodec.gen[GraphQL]
  implicit val serverCodec: JsonCodec[Server]                     = DeriveJsonCodec.gen[Server]
  implicit val jsonCodec: JsonCodec[Config]                       = DeriveJsonCodec.gen[Config]
}
