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
  def query: Option[Orchestration.ObjectTypeDefinition]                    =
    for {
      oName <- definition.collectFirst { case Orchestration.SchemaDefinition(query, _, _) => query }
      name  <- oName
      q     <- definition.collectFirst { case q @ Orchestration.ObjectTypeDefinition(`name`, _) => q }
    } yield q
}

object Orchestration {
  sealed trait Definition

  final case class ObjectTypeDefinition(name: String, fields: List[FieldDefinition])           extends Definition
  final case class InputObjectTypeDefinition(name: String, fields: List[InputValueDefinition]) extends Definition
  final case class InputValueDefinition(name: String, ofType: Type, defaultValue: Option[DynamicValue])

  final case class FieldDefinition(
    name: String,
    args: List[InputValueDefinition] = Nil,
    ofType: Type,
    resolver: Remote[Context] => Remote[DynamicValue]
  )

  final case class SchemaDefinition(
    query: Option[String] = None,
    mutation: Option[String] = None,
    subscription: Option[String] = None
  ) extends Definition

  sealed trait Type
  final case class NamedType(name: String, nonNull: Boolean) extends Type
  final case class ListType(ofType: Type, nonNull: Boolean)  extends Type

  implicit val schema: Schema[Orchestration] = DeriveSchema.gen[Orchestration]
}
