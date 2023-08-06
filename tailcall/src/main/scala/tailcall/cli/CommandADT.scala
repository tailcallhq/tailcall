package tailcall.cli

import tailcall.runtime.model.{ConfigFormat, Digest}
import zio.http.URL

import java.nio.file.Path

sealed trait CommandADT extends Serializable with Product

object CommandADT {
  sealed trait SourceFormat {
    self =>
    def name: String =
      self match {
        case SourceFormat.Postman                  => "postman"
        case SourceFormat.SchemaDefinitionLanguage => "sdl"
      }

    def named: (String, SourceFormat) = name -> self
  }

  sealed trait TargetFormat {
    self =>
    def name: String =
      self match {
        case TargetFormat.Config(fmt) => fmt match {
            case ConfigFormat.JSON    => "config-json"
            case ConfigFormat.YML     => "config-yaml"
            case ConfigFormat.GRAPHQL => "config-graphql"
          }
        case TargetFormat.JsonLines   => "json-lines"
      }

    def named: (String, TargetFormat) = name -> self
  }

  final case class BlueprintOptions(blueprint: Boolean, endpoints: Boolean, schema: Boolean)

  final case class Check(config: ::[Path], url: Option[URL], nPlusOne: Boolean, options: BlueprintOptions)
      extends CommandADT

  final case class ServerStart(
    port: Int,
    globalResponseTimeout: Int,
    enableTracing: Boolean,
    slowQueryDuration: Option[Int],
    database: Option[DBConfig],
    persistedQueries: Boolean,
    allowedHeaders: Set[String],
    file: Option[Path],
  ) extends CommandADT

  final case class DBConfig(host: String, port: Int, username: Option[String], password: Option[String])

  final case class Generate(
    files: ::[Path],
    sourceFormat: SourceFormat,
    targetFormat: TargetFormat,
    write: Option[Path],
  ) extends CommandADT

  object SourceFormat {
    case object Postman                  extends SourceFormat
    case object SchemaDefinitionLanguage extends SourceFormat
  }

  object TargetFormat {
    final case class Config(fmt: ConfigFormat) extends TargetFormat
    case object JsonLines                      extends TargetFormat
  }
}
