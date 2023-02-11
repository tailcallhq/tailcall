package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

final case class TGraph(
  operation: TGraph.Operation,
  connections: List[(String, List[(String, TGraph.Resolver)])]
)

object TGraph {
  type Resolver = Remote[Context] => Remote[DynamicValue]
  sealed trait Operation
  object Operation {
    case object Query        extends Operation
    case object Mutation     extends Operation
    case object Subscription extends Operation
  }

  def query(connections: (String, List[(String, Resolver)])*): TGraph =
    TGraph(Operation.Query, connections.toList)
}
