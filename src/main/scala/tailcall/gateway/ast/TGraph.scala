package tailcall.gateway.ast

import zio.schema.DynamicValue
import tailcall.gateway.remote.Remote

final case class TGraph(operation: TGraph.Operation, connections: List[TGraph.Connection])

object TGraph {

  sealed trait Result {
    self =>
    def ++(other: Result): Result = Result.Combine(self, other)
  }

  object Result {
    final case class Succeed(value: DynamicValue) extends Result
    case object Empty                             extends Result
    final case class Fail(cause: String)          extends Result
    final case class Delete(key: String)          extends Result
    final case class Upsert(
      key: String,
      value: DynamicValue,
      update: Remote[DynamicValue => DynamicValue],
      in: TSchema,
      output: TSchema
    ) extends Result

    final case class Combine(self: Result, other: Result) extends Result
  }

  case class Arguments(values: List[(String, DynamicValue)])

  case class Context(value: Result, args: Arguments, parent: Option[Context])

  case class TResolver(resolve: (Remote[(Arguments, Context) => Result]))

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
