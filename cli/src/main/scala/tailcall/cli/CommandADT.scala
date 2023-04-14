package tailcall.cli

import tailcall.runtime.model.Digest
import tailcall.runtime.service.DSLFormat
import zio.http.URL

import java.nio.file.Path

sealed trait CommandADT extends Serializable with Product

object CommandADT {
  final case class BlueprintOptions(blueprint: Boolean, endpoints: Boolean, schema: Boolean)
  final case class Check(config: ::[Path], url: Option[URL], options: BlueprintOptions) extends CommandADT
  final case class Generate(
    files: ::[Path],
    sourceFormat: SourceFormat,
    targetFormat: TargetFormat,
    write: Option[Path],
  ) extends CommandADT
  final case class Remote(server: URL, command: Remote.Command)                         extends CommandADT
  object Remote {
    sealed trait Command
    final case class Publish(config: ::[Path])                       extends Command
    final case class Drop(digest: Digest)                            extends Command
    final case class ListAll(offset: Int, limit: Int)                extends Command
    final case class Show(digest: Digest, options: BlueprintOptions) extends Command
  }

  sealed trait SourceFormat {
    self =>
    def name: String =
      self match {
        case SourceFormat.Postman                  => "postman"
        case SourceFormat.SchemaDefinitionLanguage => "sdl"
      }

    def named: (String, SourceFormat) = name -> self
  }
  object SourceFormat       {
    case object Postman                  extends SourceFormat
    case object SchemaDefinitionLanguage extends SourceFormat
  }

  sealed trait TargetFormat {
    self =>
    def name: String =
      self match {
        case TargetFormat.Config(fmt) => fmt match {
            case DSLFormat.JSON    => "config-json"
            case DSLFormat.YML     => "config-yaml"
            case DSLFormat.GRAPHQL => "config-graphql"
          }
        case TargetFormat.JsonLines   => "json-lines"
      }

    def named: (String, TargetFormat) = name -> self
  }

  object TargetFormat {
    final case class Config(fmt: DSLFormat) extends TargetFormat
    case object JsonLines                   extends TargetFormat
  }
}
