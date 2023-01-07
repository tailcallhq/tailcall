package tailcall.gateway.adt

import tailcall.gateway.adt.Config._
import zio.Chunk
import zio.json.{DeriveJsonCodec, JsonCodec, jsonDiscriminator, jsonHint}
import zio.parser.Syntax

final case class Config(
  version: String = "1.0.0",
  server: Server,
  schemas: List[Schema] = Nil,
  endpoints: List[Endpoint],
  graphQL: GraphQL = GraphQL(),
)

object Config {

  final case class Server(baseURL: String)
  final case class Endpoint(name: String, http: Http, input: Schema = Schema.Null, output: Schema)
  final case class Http(path: Route, method: Method = Method.GET)
  final case class GraphQL()

  @jsonDiscriminator("type")
  sealed trait Schema
  object Schema {

    sealed trait Scalar extends Schema

    @jsonHint("String")
    case object Str extends Scalar

    @jsonHint("Integer")
    case object Int extends Scalar

    @jsonHint("ID")
    case object Id extends Scalar

    @jsonHint("null")
    case object Null extends Scalar

    @jsonHint("object")
    final case class Obj(fields: List[Field]) extends Schema

    @jsonHint("array")
    final case class Arr(items: Schema) extends Schema

    final case class Field(name: String, schema: Schema, required: Boolean = false)
  }

  final case class Route(segments: List[Route.Segment])
  object Route {
    sealed trait Segment
    object Segment {
      final case class Literal(value: String) extends Segment
      final case class Param(value: String)   extends Segment
    }

    object syntax {
      val segment = Syntax
        .alphaNumeric
        .repeat
        .transform[String](_.asString, s => Chunk.fromIterable(s))

      val param =
        (
          Syntax.string("${", ()) ~ segment ~ Syntax.char('}')
        ).transform[Segment.Param](Segment.Param, _.value)

      val literal = segment.transform[Segment.Literal](Segment.Literal, _.value)

      val segmentChunk =
        (
          Syntax.char('/') ~ (literal.widen[Segment] | param.widen[Segment])
        ).repeat

      val route = segmentChunk
        .transform[Route](chunk => Route(chunk.toList), route => Chunk.from(route.segments))

    }

    def decode(string: String): Either[String, Route] =
      syntax.route.parseString(string) match {
        case Left(_)      =>
          Left(s"Invalid route: ${string}")
        case Right(value) =>
          Right(value)
      }

    def encode(route: Route): Either[String, String] = syntax.route.asPrinter.printString(route)

  }

  sealed trait Method
  object Method {
    case object GET    extends Method
    case object POST   extends Method
    case object PUT    extends Method
    case object DELETE extends Method

    def encode(method: Method): String                 =
      method match {
        case Method.GET    =>
          "GET"
        case Method.POST   =>
          "POST"
        case Method.PUT    =>
          "PUT"
        case Method.DELETE =>
          "DELETE"
      }
    def decode(method: String): Either[String, Method] =
      method match {
        case "GET"    =>
          Right(Method.GET)
        case "POST"   =>
          Right(Method.POST)
        case "PUT"    =>
          Right(Method.PUT)
        case "DELETE" =>
          Right(Method.DELETE)
        case name     =>
          Left("Unknown method: " + name)
      }
  }

  /**
   * Json Codecs
   *
   * TODO: This should only be done once, not for every
   * instance of Config
   */

  implicit lazy val fieldSchema: JsonCodec[Schema.Field]    = DeriveJsonCodec.gen[Schema.Field]
  implicit lazy val schemaCodec: zio.json.JsonCodec[Schema] = zio.json.DeriveJsonCodec.gen[Schema]
  implicit lazy val methodCodec: JsonCodec[Method]          = JsonCodec[String]
    .transformOrFail(Method.decode, Method.encode)
  implicit lazy val httpCodec: JsonCodec[Http]              = DeriveJsonCodec.gen[Http]
  implicit lazy val globalHttpCodec: JsonCodec[Server]      = DeriveJsonCodec.gen[Server]
  implicit lazy val endpointCodec: JsonCodec[Endpoint]      = DeriveJsonCodec.gen[Endpoint]
  implicit lazy val graphQLCodec: JsonCodec[GraphQL]        = DeriveJsonCodec.gen[GraphQL]
  implicit lazy val routeCodec: JsonCodec[Route]            = JsonCodec[String].transformOrFail(
    Route.decode,
    route => Route.encode(route).getOrElse(throw new RuntimeException("Invalid Route")),
  )
  implicit lazy val configCodec: JsonCodec[Config]          = DeriveJsonCodec.gen[Config]

}
