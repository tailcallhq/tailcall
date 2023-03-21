package tailcall.cli

import tailcall.runtime.ast.Digest

import java.nio.file.Path

sealed trait CommandADT extends Serializable with Product

object CommandADT {
  final case class Deploy(orc: Path)                          extends CommandADT
  final case class Drop(digest: Digest)                       extends CommandADT
  final case class GetAll(index: Int, offset: Int)            extends CommandADT
  final case class GetOne(digest: Digest)                     extends CommandADT
  final case class Compile(input: Path, output: Option[Path]) extends CommandADT
  final case class GraphQLSchema(blueprint: Path)             extends CommandADT
}
