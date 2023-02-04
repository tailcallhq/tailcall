package tailcall.gateway.ast

import tailcall.gateway.remote.TResolve
import zio.schema.DynamicValue

final case class TGraph(operation: TGraph.Operation, connections: List[TGraph.Connection])

object TGraph {
  case class Arguments(values: List[(String, DynamicValue)])

  case class Context(value: Either[String, DynamicValue], args: Arguments, parent: Option[Context])

  final case class Connection(
    name: String,
    from: TSchema,
    to: TSchema,
    arg: List[(String, TSchema)],
    resolver: TResolve[Context, String, DynamicValue]
  )

  sealed trait Operation
  object Operation {
    case object Query        extends Operation
    case object Mutation     extends Operation
    case object Subscription extends Operation
  }
}
