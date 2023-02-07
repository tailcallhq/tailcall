package tailcall.gateway.ast

import zio.schema._

/**
 * The core AST to represent API orchestration. It takes in
 * an input of type A and performs a series of steps to
 * produce an output of type B.
 */
sealed trait Orc {
  self =>
  def >>>(other: Orc): Orc    = Orc.OrchPipe(self, other)
  def ++(other: Orc): Orc     = Orc.OrchConcat(self, other)
  def pipe(other: Orc): Orc   = self >>> other
  def concat(other: Orc): Orc = self ++ other
}

object Orc {
  final case class OrchValue(dynamic: DynamicValue)          extends Orc
  final case class OrchList(list: List[Orc])                extends Orc
  final case class OrchObject(fields: Map[String, Orc])     extends Orc
  final case class OrchEndpoint(endpoint: Endpoint)          extends Orc
  final case class OrchPipe(left: Orc, right: Orc)         extends Orc
  final case class OrchPath(path: List[String])              extends Orc
  final case class OrchApplySpec(path: List[(String, Orc)]) extends Orc
  final case class OrchConcat(left: Orc, right: Orc)       extends Orc
  final case class OrchBatch(endpoint: Orc, groupBy: Orc)  extends Orc

  def obj(fields: (String, Orc)*): Orc         = OrchObject(fields.toMap)
  def endpoint(endpoint: Endpoint): Orc         = OrchEndpoint(endpoint)
  def spec(path: (String, Orc)*): Orc          = OrchApplySpec(path.toList)
  def concat(left: Orc, right: Orc): Orc      = OrchConcat(left, right)
  def path(path: String*): Orc                  = OrchPath(path.toList)
  def context(path: String*): Orc               = OrchPath("context" :: path.toList)
  def batch(endpoint: Orc, groupBy: Orc): Orc = OrchBatch(endpoint, groupBy)

  implicit val schema: Schema[Orc] = DeriveSchema.gen[Orc]
}
