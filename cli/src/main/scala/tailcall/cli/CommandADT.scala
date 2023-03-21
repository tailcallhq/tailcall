package tailcall.cli

import tailcall.runtime.ast.Digest
import zio.http.URL

import java.nio.file.Path

sealed trait CommandADT extends Serializable with Product

object CommandADT {
  final case class Check(config: Path, url: Option[URL])        extends CommandADT
  final case class Remote(server: URL, command: Remote.Command) extends CommandADT
  object Remote {
    sealed trait Command
    final case class Publish(config: Path)                           extends Command
    final case class Drop(digest: Digest)                            extends Command
    final case class ShowAll(offset: Int, limit: Int)                extends Command
    final case class ShowOne(digest: Digest, graphQLSchema: Boolean) extends Command
  }
}
