package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

final case class Orc(
  query: List[(String, List[(String, Orc.Resolver)])] = Nil,
  mutation: List[(String, List[(String, Orc.Resolver)])] = Nil,
  subscription: List[(String, List[(String, Orc.Resolver)])] = Nil
) {
  def ++(other: Orc): Orc =
    Orc(
      query = query ++ other.query,
      mutation = mutation ++ other.mutation,
      subscription = subscription ++ other.subscription
    )
}

object Orc {
  type Resolver = Remote[Context] => Remote[DynamicValue]

  def query(connections: (String, List[(String, Resolver)])*): Orc = Orc(query = connections.toList)

  def mutation(connections: (String, List[(String, Resolver)])*): Orc =
    Orc(mutation = connections.toList)

  def subscription(connections: (String, List[(String, Resolver)])*): Orc =
    Orc(subscription = connections.toList)
}
