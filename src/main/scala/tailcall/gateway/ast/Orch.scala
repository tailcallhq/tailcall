package tailcall.gateway.ast

import zio.schema._

/**
 * The core AST to represent API orchestration. It takes in
 * an input of type A and performs a series of steps to
 * produce an output of type B.
 */
sealed trait Orch {
  self =>
  def >>>(other: Orch): Orch    = Orch.OrchPipe(self, other)
  def ++(other: Orch): Orch     = Orch.OrchConcat(self, other)
  def pipe(other: Orch): Orch   = self >>> other
  def concat(other: Orch): Orch = self ++ other
}

object Orch {
  final case class OrchValue(dynamic: DynamicValue)          extends Orch
  final case class OrchList(list: List[Orch])                extends Orch
  final case class OrchObject(fields: Map[String, Orch])     extends Orch
  final case class OrchEndpoint(endpoint: Endpoint)          extends Orch
  final case class OrchPipe(left: Orch, right: Orch)         extends Orch
  final case class OrchPath(path: List[String])              extends Orch
  final case class OrchApplySpec(path: List[(String, Orch)]) extends Orch
  final case class OrchConcat(left: Orch, right: Orch)       extends Orch
  final case class OrchBatch(endpoint: Orch, groupBy: Orch)  extends Orch

  def obj(fields: (String, Orch)*): Orch         = OrchObject(fields.toMap)
  def endpoint(endpoint: Endpoint): Orch         = OrchEndpoint(endpoint)
  def spec(path: (String, Orch)*): Orch          = OrchApplySpec(path.toList)
  def concat(left: Orch, right: Orch): Orch      = OrchConcat(left, right)
  def path(path: String*): Orch                  = OrchPath(path.toList)
  def context(path: String*): Orch               = OrchPath("context" :: path.toList)
  def batch(endpoint: Orch, groupBy: Orch): Orch = OrchBatch(endpoint, groupBy)

  implicit val schema: Schema[Orch] = DeriveSchema.gen[Orch]
}
