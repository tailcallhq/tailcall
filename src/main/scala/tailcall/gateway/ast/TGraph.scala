package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

final case class TGraph(operation: TGraph.Operation, connections: List[TGraph.Connection])

object TGraph {

  sealed trait Result
  object Result {
    final case class Succeed(value: DynamicValue) extends Result
    final case class Fail(cause: String)          extends Result
  }

  case class Arguments(values: List[(String, DynamicValue)])

  case class Context(value: Result, args: Arguments, parent: Option[Context])

  sealed trait TResolver
  object TResolver {
    case class FromFunction(resolve: (Remote[(Arguments, Context) => Result]))
    case class FromEndpoint(endpoint: Endpoint)
  }

  final case class Connection(
    name: String,
    from: TSchema,
    to: TSchema,
    arg: List[(String, TSchema)],
    resolver: TResolver
  )

  sealed trait Operation
  object Operation {
    case object Query        extends Operation
    case object Mutation     extends Operation
    case object Subscription extends Operation
  }
}
