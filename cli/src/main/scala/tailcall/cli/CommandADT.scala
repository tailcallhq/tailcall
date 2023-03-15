package tailcall.cli

import java.nio.file.Path

sealed trait CommandADT extends Serializable with Product

object CommandADT {
  final case class Deploy(digest: String, endpoint: String) extends CommandADT
  final case class Validate(file: Path)                     extends CommandADT
}
