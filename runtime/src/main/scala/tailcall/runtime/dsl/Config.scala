package tailcall.runtime.dsl

import tailcall.runtime.ast._
import tailcall.runtime.dsl.Config._
import tailcall.runtime.http.Method
import tailcall.runtime.service.ConfigFileIO
import tailcall.runtime.transcoder.Transcoder
import zio.ZIO
import zio.json._
import zio.json.ast.Json

import java.io.File
import java.net.URL

final case class Config(version: Int = 0, server: Server = Server(), graphQL: GraphQL = GraphQL()) {
  self =>
  def toBlueprint: Blueprint = toBlueprint()

  def toBlueprint(encodeSteps: Boolean = false): Blueprint = Transcoder.toBlueprint(self, encodeSteps = encodeSteps).get

  def mergeRight(other: Config): Config = {
    Config(
      version = other.version,
      server = self.server.mergeRight(other.server),
      graphQL = self.graphQL.mergeRight(other.graphQL),
    )
  }

  def compress: Config = self.copy(graphQL = self.graphQL.compress)
}

object Config {
  final case class Server(baseURL: Option[URL] = None) {
    self =>
    def mergeRight(other: Server): Server = Server(baseURL = other.baseURL.orElse(self.baseURL))
  }

  final case class RootSchema(query: Option[String] = None, mutation: Option[String] = None)
  final case class GraphQL(schema: RootSchema = RootSchema(), types: Map[String, Map[String, Field]] = Map.empty) {
    self =>
    def mergeRight(other: GraphQL): GraphQL =
      GraphQL(
        schema = RootSchema(
          query = other.schema.query.orElse(self.schema.query),
          mutation = other.schema.mutation.orElse(self.schema.mutation),
        ),
        types = self.types ++ other.types,
      )

    def compress: GraphQL                                                                  =
      self.copy(types = self.types.map { case (k, v) => k -> v.map { case (k, v) => (k, v.compress) } })
    def withSchema(query: Option[String] = None, mutation: Option[String] = None): GraphQL =
      copy(schema = RootSchema(query, mutation))
    def withType(name: String, fields: Map[String, Field]): GraphQL = copy(types = types + (name -> fields))
  }

  // TODO: Field and Argument can be merged
  final case class Field(
    @jsonField("type") typeOf: String,
    isList: Option[Boolean] = None,
    isRequired: Option[Boolean] = None,
    steps: Option[List[Step]] = None,
    args: Option[Map[String, Argument]] = None,
  ) {
    self =>
    def asList: Field                                     = copy(isList = Option(true))
    def asRequired: Field                                 = copy(isRequired = Option(true))
    def withArguments(args: Map[String, Argument]): Field = copy(args = Option(args))
    def withSteps(steps: List[Step]): Field               = copy(steps = Option(steps))
    def apply(args: (String, Argument)*): Field           = copy(args = Option(args.toMap))
    def compress: Field                                   = {
      val isList = self.isList match {
        case Some(true) => Some(true)
        case _          => None
      }

      val isRequired = self.isRequired match {
        case Some(true) => Some(true)
        case _          => None
      }

      val steps = self.steps match {
        case Some(steps) if steps.nonEmpty =>
          Option(steps.map {
            case step @ Step.Http(_, _, _, _) =>
              val noOutputHttp = step.withOutput(None)
              if (step.method contains Method.GET) noOutputHttp.withMethod(None) else noOutputHttp
            case step                         => step
          })
        case _                             => None
      }

      val args = self.args match {
        case Some(args) if args.nonEmpty => Some(args.map { case (k, v) => (k, v.compress) })
        case _                           => None
      }

      self.copy(isList = isList, isRequired = isRequired, steps = steps, args = args)
    }
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
    @jsonHint("http")
    final case class Http(
      path: Path,
      method: Option[Method] = None,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None,
    ) extends Step {
      def withOutput(output: Option[TSchema]): Http = copy(output = output)
      def withInput(input: Option[TSchema]): Http   = copy(input = input)
      def withMethod(method: Option[Method]): Http  = copy(method = method)
    }

    object Http {
      def fromEndpoint(endpoint: Endpoint): Http =
        Http(path = endpoint.path, method = Option(endpoint.method), input = endpoint.input, output = endpoint.output)
    }

    @jsonHint("const")
    final case class Constant(json: Json) extends Step
    object Constant {
      implicit val codec: JsonCodec[Constant] = JsonCodec(Json.encoder, Json.decoder).transform(Constant(_), _.json)
    }

    @jsonHint("objectPath")
    final case class ObjPath(map: Map[String, List[String]]) extends Step
    object ObjPath {
      def apply(map: (String, List[String])*): ObjPath = ObjPath(map.toMap)
      implicit val codec: JsonCodec[ObjPath] = JsonCodec[Map[String, List[String]]].transform(ObjPath(_), _.map)
    }
  }

  final case class Argument(
    @jsonField("type") typeOf: String,
    isList: Option[Boolean] = None,
    isRequired: Option[Boolean] = None,
  ) {
    self =>
    def asList: Argument     = self.copy(isList = Option(true))
    def asRequired: Argument = self.copy(isRequired = Option(true))
    def compress: Argument   = {
      val isList = self.isList match {
        case Some(true) => Some(true)
        case _          => None
      }

      val isRequired = self.isRequired match {
        case Some(true) => Some(true)
        case _          => None
      }

      self.copy(isList = isList, isRequired = isRequired)
    }
  }

  object Argument {
    val string: Argument = Argument("String")
    val int: Argument    = Argument("Int")
    val bool: Argument   = Argument("Boolean")
  }

  def fromFile(file: File): ZIO[ConfigFileIO, Throwable, Config] = ConfigFileIO.readFile(file)

  /**
   * Json Codecs
   *
   * TODO: This should only be done once, not for every
   * instance of Config. This is done currently because if
   * we create a jsonCodec from Schema, internally Maps are
   * stored as chunks and the outputted json looks like a
   * list of tuples.
   */

  implicit val urlCodec: JsonCodec[URL]                     = JsonCodec[String].transformOrFail[URL](
    string =>
      try Right(new URL(string))
      catch { case _: Throwable => Left(s"Malformed url: ${string}") },
    _.toString,
  )
  implicit val operationCodec: JsonCodec[Step]              = DeriveJsonCodec.gen[Step]
  implicit val inputTypeCodec: JsonCodec[Argument]          = DeriveJsonCodec.gen[Argument]
  implicit val fieldDefinitionCodec: JsonCodec[Field]       = DeriveJsonCodec.gen[Field]
  implicit val schemaDefinitionCodec: JsonCodec[RootSchema] = DeriveJsonCodec.gen[RootSchema]
  implicit val graphQLCodec: JsonCodec[GraphQL]             = DeriveJsonCodec.gen[GraphQL]
  implicit val serverCodec: JsonCodec[Server]               = DeriveJsonCodec.gen[Server]
  implicit val jsonCodec: JsonCodec[Config]                 = DeriveJsonCodec.gen[Config]
}
