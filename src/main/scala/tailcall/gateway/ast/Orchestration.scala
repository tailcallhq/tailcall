package tailcall.gateway.ast

import caliban.GraphQL
import tailcall.gateway.remote.Remote
import tailcall.gateway.service.OrchestrationGraphQLGenerator
import zio.ZIO
import zio.schema.{DeriveSchema, DynamicValue, Schema}

final case class Orchestration(definition: List[Orchestration.Definition]) {
  self =>

  def toGraphQL: ZIO[OrchestrationGraphQLGenerator, Nothing, GraphQL[Any]] =
    OrchestrationGraphQLGenerator.toGraphQL(self)
}

// scalafmt: {maxColumn = 240}
object Orchestration {
  // TODO: create a common type for Object
  // TODO: drop non-null fields
  // TODO: create a common type for input and field use phantom types

  sealed trait Definition

  final case class ObjectTypeDefinition(name: String, fields: List[FieldDefinition])           extends Definition
  final case class InputObjectTypeDefinition(name: String, fields: List[InputValueDefinition]) extends Definition
  final case class InputValueDefinition(name: String, ofType: Type, defaultValue: Option[DynamicValue])

  final case class FieldDefinition(name: String, args: List[InputValueDefinition] = Nil, ofType: Type, resolver: Remote[Context] => Remote[DynamicValue])

  final case class SchemaDefinition(query: Option[String] = None, mutation: Option[String] = None, subscription: Option[String] = None) extends Definition

  sealed trait Type
  final case class NamedType(name: String, nonNull: Boolean) extends Type
  final case class ListType(ofType: Type, nonNull: Boolean)  extends Type

  implicit val schema: Schema[Orchestration] = DeriveSchema.gen[Orchestration]
}
