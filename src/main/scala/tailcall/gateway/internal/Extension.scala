package tailcall.gateway.internal

import tailcall.gateway.dsl.json.Config
import zio.json.yaml._
import zio.json.{DecoderOps, _}
import zio.{Task, ZIO}

sealed private[tailcall] trait Extension {
  self =>

  def name: String                            =
    this match {
      case Extension.JSON => "json"
      case Extension.YML  => "yml"
    }
  def decode[A](string: String): Task[Config] =
    ZIO
      .fromEither(self match {
        case Extension.JSON => string.fromJson[Config]
        case Extension.YML  => string.fromYaml[Config]
      })
      .mapError(new RuntimeException(_))

  def encode(config: Config): Task[String] =
    ZIO
      .fromEither(self match {
        case Extension.JSON => Right(config.toJsonPretty)
        case Extension.YML  =>
          config.toYaml(YamlOptions.default.copy(sequenceIndentation = 0))
      })
      .mapError(new RuntimeException(_))
}

object Extension {
  case object JSON extends Extension
  case object YML  extends Extension

  def detect(name: String): Task[Extension] = {
    if (name.endsWith(".json")) ZIO.succeed(JSON)
    else if (name.endsWith(".yml")) ZIO.succeed(YML)
    else ZIO.fail(new RuntimeException(s"Unsupported file format: ${name}"))
  }
}
