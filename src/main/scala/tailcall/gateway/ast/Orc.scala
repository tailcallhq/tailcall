package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

final case class Orc(
  operation: Orc.Operation,
  connections: List[(String, List[(String, Orc.Resolver)])]
)

object Orc {
  type Resolver = Remote[Context] => Remote[DynamicValue]
  sealed trait Operation
  object Operation {
    case object Query        extends Operation
    case object Mutation     extends Operation
    case object Subscription extends Operation
  }

  def query(connections: (String, List[(String, Resolver)])*): Orc =
    Orc(Operation.Query, connections.toList)
}
