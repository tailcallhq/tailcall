package tailcall.runtime.service

import caliban.parsing.Parser
import tailcall.runtime.model.Config
import tailcall.runtime.transcoder.Transcoder
import zio.json._
import zio.json.yaml._
import zio.{IO, ZIO}

sealed trait ConfigFormat {
  self =>

  def ext: String =
    self match {
      case ConfigFormat.JSON    => "json"
      case ConfigFormat.YML     => "yml"
      case ConfigFormat.GRAPHQL => "graphql"
    }

  def encode(config: Config): IO[String, String] =
    self match {
      case ConfigFormat.JSON    => ZIO.succeed(config.toJsonPretty)
      case ConfigFormat.YML     => ZIO.fromEither(config.toYaml(YamlOptions.default.copy(sequenceIndentation = 0)))
      case ConfigFormat.GRAPHQL => Transcoder.toGraphQLConfig(config).toZIO.mapError(_.mkString(", "))
    }

  // TODO: doesn't need IO
  def decode(string: String): IO[String, Config] =
    (self match {
      case ConfigFormat.JSON    => ZIO.fromEither(string.fromJson[Config])
      case ConfigFormat.YML     => ZIO.fromEither(string.fromYaml[Config])
      case ConfigFormat.GRAPHQL => Parser.parseQuery(string).mapError(_.msg)
          .flatMap(Transcoder.toConfig(_).toZIO.mapError(_.mkString(", ")))
    }).map(_.compress)

  def endsWith(file: String): Boolean = file.endsWith(s".${ext}")
}

object ConfigFormat {
  case object JSON    extends ConfigFormat
  case object YML     extends ConfigFormat
  case object GRAPHQL extends ConfigFormat

  def all: List[ConfigFormat] = List(JSON, YML, GRAPHQL)

  def detect(name: String): IO[String, ConfigFormat] =
    all.find(_.endsWith(name)).fold[IO[String, ConfigFormat]](ZIO.fail(s"Unsupported file: ${name}"))(ZIO.succeed(_))
}
