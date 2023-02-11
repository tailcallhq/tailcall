package tailcall.gateway.ast

import zio.schema._

/**
 * Orc is virtually a function from A to B. It doesn't have
 * any type information because the type verification only
 * happens at runtime.
 */
sealed trait Orc[-A, +B] {
  self =>
  final def <<<[A0](other: Orc[A0, A]): Orc[A0, B]     = other >>> self
  final def >>>[C](other: Orc[B, C]): Orc[A, C]        = Orc.Pipe(self, other)
  final def pipe[C](other: Orc[B, C]): Orc[A, C]       = self >>> other
  final def compose[A0](other: Orc[A0, A]): Orc[A0, B] = self <<< other
}

object Orc {
  final case class FromEndpoint(endpoint: Endpoint) extends Orc[DynamicValue, DynamicValue]
  final case class Pipe[A, B, C](left: Orc[A, B], right: Orc[B, C]) extends Orc[A, C]
  final case class Select(fields: List[(String, List[String])])
      extends Orc[DynamicValue, DynamicValue]

  def endpoint(endpoint: Endpoint): Orc[DynamicValue, DynamicValue] = FromEndpoint(endpoint)
  def select(fields: (String, List[String])*): Orc[DynamicValue, DynamicValue] =
    Select(fields.toList)

  // implicit def schema[A, B]: Schema[Orc[A, B]] = DeriveSchema.gen[Orc[A, B]]
}
