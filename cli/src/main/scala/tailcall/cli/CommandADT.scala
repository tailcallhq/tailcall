package tailcall.cli

import tailcall.runtime.ast.Digest

import java.net.InetAddress
import java.nio.file.Path

sealed trait CommandADT extends Serializable with Product

object CommandADT {
  final case class Remote(address: InetAddress, command: Remote.Command) extends CommandADT
  object Remote {
    sealed trait Command
    final case class Deploy(orc: Path)          extends Command
    final case class Drop(digest: Digest)       extends Command
    final case class Activate(digest: Digest)   extends Command
    final case class Deactivate(digest: Digest) extends Command
    final case object List                      extends Command
    final case class Info(digest: Digest)       extends Command
  }

  final case class Compile(input: Path, output: Option[Path]) extends CommandADT
  final case class GraphQLSchema(blueprint: Path)             extends CommandADT
}
