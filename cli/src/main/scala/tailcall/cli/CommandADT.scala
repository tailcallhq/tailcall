package tailcall.cli

import tailcall.runtime.ast.Digest
import zio.http.URL

import java.nio.file.Path

sealed trait CommandADT extends Serializable with Product

object CommandADT {
  final case class BlueprintOptions(blueprint: Boolean, endpoints: Boolean, schema: Boolean)
  final case class Check(config: Path, url: Option[URL], options: BlueprintOptions) extends CommandADT
  final case class Remote(server: URL, command: Remote.Command)                     extends CommandADT
  object Remote {
    sealed trait Command
    final case class Publish(config: Path)                           extends Command
    final case class Drop(digest: Digest)                            extends Command
    final case class ListAll(offset: Int, limit: Int)                extends Command
    final case class Show(digest: Digest, options: BlueprintOptions) extends Command
  }
}
