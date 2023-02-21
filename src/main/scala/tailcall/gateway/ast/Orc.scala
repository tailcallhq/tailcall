package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema.{DeriveSchema, DynamicValue, Schema}

sealed trait Orc {
  self =>

}

object Orc {
  final case class OrcValue(dynamicValue: DynamicValue)              extends Orc
  final case class OrcObject(name: String, fields: Map[String, Orc]) extends Orc
  final case class OrcList(values: List[Orc])                        extends Orc
  final case class OrcFunction(fun: Remote[Context] => Remote[Orc])  extends Orc
  final case class OrcRef(name: String)                              extends Orc

  def value[A](a: A)(implicit schema: Schema[A]): Orc = OrcValue(schema.toDynamic(a))

  def obj(name: String)(fields: (String, Orc)*): Orc = OrcObject(name, fields.toMap)

  def list(values: Orc*): Orc = OrcList(values.toList)

  def function(fun: Remote[Context] => Remote[Orc]): Orc = OrcFunction(Remote.bind(fun))

  def ref(name: String): Orc = OrcRef(name)

  implicit lazy val schema: Schema[Orc] = DeriveSchema.gen[Orc]
}
