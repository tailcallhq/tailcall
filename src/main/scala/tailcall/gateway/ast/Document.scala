package tailcall.gateway.ast

import caliban.GraphQL
import tailcall.gateway.remote.Remote
import tailcall.gateway.service.GraphQLGenerator
import zio.ZIO
import zio.schema.{DeriveSchema, DynamicValue, Schema}

/**
 * Document is an intermediate representation of a GraphQL
 * document. It has two features — 1. It is serializable and
 * 2. It has logic to resolve fields into actual values.
 *
 * IMPORTANT: we should keep this as close to Caliban's AST
 * as much as possible. The idea is that sometimes we might
 * need some changes in Caliban's AST, for eg: we need to
 * generate a ZIO Schema of the Caliban AST. This is
 * currently not possible because the case classes are not
 * final. Instead of opening a PR in Caliban, we can just
 * make the changes here and then use the modified AST. The
 * other reason is that our IR design isn't very clearly
 * thought out. So we will use Document as a playground to
 * try out different IRs. Document supports each and every
 * feature that GraphQL has to offer so we keep it until IR
 * is clearly defined. Once the IR is ready we will directly
 * compile IR to Caliban's Step ADT.
 */
final case class Document(definition: List[Document.Definition]):
  self =>

  def toGraphQL: ZIO[GraphQLGenerator, Nothing, GraphQL[Any]] = GraphQLGenerator.toGraphQL(self)

// scalafmt: {maxColumn = 240}
object Document:
  // TODO: create a common type for Object
  // TODO: drop non-null fields
  // TODO: create a common type for input and field use phantom types

  sealed trait Definition

  final case class ObjectTypeDefinition(name: String, fields: List[FieldDefinition])           extends Definition
  final case class InputObjectTypeDefinition(name: String, fields: List[InputValueDefinition]) extends Definition
  final case class InputValueDefinition(name: String, ofType: Type, defaultValue: Option[DynamicValue])

  final case class FieldDefinition(name: String, args: List[InputValueDefinition] = Nil, ofType: Type, resolver: Remote[DynamicValue] => Remote[DynamicValue])

  final case class SchemaDefinition(query: Option[String] = None, mutation: Option[String] = None, subscription: Option[String] = None) extends Definition

  sealed trait Type
  final case class NamedType(name: String, nonNull: Boolean) extends Type
  final case class ListType(ofType: Type, nonNull: Boolean)  extends Type

  given Schema[Document] = DeriveSchema.gen[Document]
