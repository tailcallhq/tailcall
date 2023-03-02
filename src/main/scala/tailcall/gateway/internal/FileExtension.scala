package tailcall.gateway.internal

import tailcall.gateway.dsl.json.Config
import zio.json.yaml._
import zio.json.{DecoderOps, _}
import zio.{Task, ZIO}

sealed private[tailcall] trait FileExtension:
  self =>

  def name: String                            =
    this match
      case FileExtension.JSON => "json"
      case FileExtension.YML  => "yml"
  def decode[A](string: String): Task[Config] =
    ZIO.fromEither(self match {
      case FileExtension.JSON => string.fromJson[Config]
      case FileExtension.YML  => string.fromYaml[Config]
    }).mapError(new RuntimeException(_))

  def encode(config: Config): Task[String] =
    ZIO.fromEither(self match {
      case FileExtension.JSON => Right(config.toJsonPretty)
      case FileExtension.YML  => config.toYaml(YamlOptions.default.copy(sequenceIndentation = 0))
    }).mapError(new RuntimeException(_))

object FileExtension:
  case object JSON extends FileExtension
  case object YML  extends FileExtension

  def detect(name: String): Task[FileExtension] =
    if name.endsWith(".json") then ZIO.succeed(JSON)
    else if name.endsWith(".yml") then ZIO.succeed(YML)
    else ZIO.fail(new RuntimeException(s"Unsupported file format: ${name}"))
