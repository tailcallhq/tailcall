package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema._

/**
 * The core AST to represent API orchestration. It takes in
 * an input of type A and performs a series of steps to
 * produce an output of type B.
 */
sealed trait Orc

object Orc {
  final case class FunctionOrc(orc: Remote[Map[String, DynamicValue] => Orc]) extends Orc
  final case class ListOrc(orcs: List[Orc])                                   extends Orc
  final case class ObjectOrc(name: String, fields: Map[String, Orc])          extends Orc
  final case class EndpointOrc(endpoint: Endpoint)                            extends Orc
  final case class RemoteOrc(orc: Remote[Orc])                                extends Orc

  def obj(fields: (String, Orc)*): Orc                              = ObjectOrc("", fields.toMap)
  def list(orcs: Orc*): Orc                                         = ListOrc(orcs.toList)
  def endpoint(endpoint: Endpoint): Orc                             = EndpointOrc(endpoint)
  def function(orcs: Remote[Map[String, DynamicValue] => Orc]): Orc = FunctionOrc(orcs)
  def remote(orc: Remote[Orc]): Orc                                 = RemoteOrc(orc)

  implicit val schema: Schema[Orc] = DeriveSchema.gen[Orc]
}
