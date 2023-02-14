package tailcall.gateway.dsl.json

import tailcall.gateway.ast._
import tailcall.gateway.http.Method
import zio.json._

import Config.Operation.Transform
import Config._
final case class Config(
  version: String = "1.0.0",
  server: Server,
  graphQL: Specification = Specification(Map.empty)
)

object Config {
  final case class Server(baseURL: String)
  final case class Specification(connections: Map[String, Map[String, Connection]])
  final case class Connection(operations: List[ConfigEndpoint])
  final case class ConfigEndpoint(
    operation: Operation,
    input: Option[TSchema] = None,
    output: TSchema
  )

  @jsonDiscriminator("type")
  sealed trait Operation
  object Operation {
    @jsonHint("http")
    final case class Http(path: Path, method: Method = Method.GET, query: List[QueryParam] = Nil)
        extends Operation

    // TODO: value should not be a string
    // It should be a template string
    final case class QueryParam(name: String, value: String)

    @jsonHint("transformation")
    final case class Transformation(@jsonField("apply") transform: Transform) extends Operation

    sealed trait Transform

    @jsonHint("^identity")
    case object Identity extends Transform

    @jsonHint("^compose")
    final case class Compose(transforms: List[Transform]) extends Transform
  }

  /**
   * Json Codecs
   *
   * TODO: This should only be done once, not for every
   * instance of Config
   */

  implicit lazy val operationCodec: JsonCodec[Operation] = { DeriveJsonCodec.gen[Operation] }

  implicit lazy val transformCodec: JsonCodec[Transform] = { DeriveJsonCodec.gen[Transform] }

  implicit lazy val serverCodec: JsonCodec[Server] = { DeriveJsonCodec.gen[Server] }

  implicit lazy val endpointCodec: JsonCodec[ConfigEndpoint] = {
    DeriveJsonCodec.gen[ConfigEndpoint]
  }

  implicit lazy val connectionCodec: JsonCodec[Connection] = { DeriveJsonCodec.gen[Connection] }

  implicit lazy val specificationCodec: JsonCodec[Specification] = {
    DeriveJsonCodec.gen[Specification]
  }

  implicit lazy val configCodec: JsonCodec[Config] = { DeriveJsonCodec.gen[Config] }

  implicit lazy val queryParamCodec: JsonCodec[Operation.QueryParam] = {
    DeriveJsonCodec.gen[Operation.QueryParam]
  }
}
