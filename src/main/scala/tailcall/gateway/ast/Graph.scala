package tailcall.gateway.ast

import tailcall.gateway.lambda.DynamicEval
import tailcall.gateway.remote.Remote
import zio.Chunk
import zio.schema.codec.JsonCodec.JsonEncoder
import zio.schema.{DeriveSchema, Schema}

/**
 * A `GraphQL` represents a connection between two nodes in
 * a graph.
 */

final case class Graph(fields: List[Graph.Field]) {
  self =>
  def ++(other: Graph): Graph       = Graph(self.fields ++ other.fields)
  def ::(field: Graph.Field): Graph = Graph(field :: self.fields)
  def combine(other: Graph): Graph  = self ++ other
  def binary: Chunk[Byte]           = JsonEncoder.encode(Graph.schema, self)
  def toJson: String                = new String(binary.toArray)
}

object Graph {
  final case class Field(
    name: String,
    argType: Schema[Any],
    fromType: Schema[Any],
    toType: Schema[Any],
    executable: Remote[Context] => Remote[DynamicEval]
  ) {
    def toGraph: Graph = Graph(List(this))
  }

  def empty: Graph = Graph(Nil)

  implicit val schema = DeriveSchema.gen[Graph]
}
