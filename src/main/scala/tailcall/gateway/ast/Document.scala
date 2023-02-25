package tailcall.gateway.ast

import caliban.GraphQL
import caliban.introspection.adt.__Type
import caliban.schema.Step
import tailcall.gateway.remote.Remote
import tailcall.gateway.service.{DocumentGraphQLGenerator, DocumentStepGenerator}
import zio.ZIO
import zio.query.ZQuery
import zio.schema.{DeriveSchema, DynamicValue, Schema}

final case class Document(definition: List[Document.Definition]) {
  self =>

  def toGraphQL: ZIO[DocumentGraphQLGenerator, Nothing, GraphQL[Any]] = DocumentGraphQLGenerator.toGraphQL(self)
  def query: Option[Document.Definition.ObjectTypeDefinition]         =
    for {
      oName <- definition.collectFirst { case Document.Definition.SchemaDefinition(query, _, _) => query }
      name  <- oName
      q     <- definition.collectFirst { case q @ Document.Definition.ObjectTypeDefinition(`name`, _) => q }
    } yield q
}

object Document {
  sealed trait Definition
  object Definition {

    final case class ObjectTypeDefinition(name: String, fields: List[FieldDefinition])           extends Definition
    final case class InputObjectTypeDefinition(name: String, fields: List[InputValueDefinition]) extends Definition
    final case class InputValueDefinition(name: String, ofType: Type, defaultValue: Option[DynamicValue])
        extends Definition
    final case class FieldDefinition(
      name: String,
      args: List[InputValueDefinition] = Nil,
      ofType: Type,
      resolver: FieldResolver
    ) extends Definition

    final case class SchemaDefinition(
      query: Option[String] = None,
      mutation: Option[String] = None,
      subscription: Option[String] = None
    ) extends Definition
  }

  sealed trait Type
  object Type {
    final case class NamedType(name: String, nonNull: Boolean) extends Type
    final case class ListType(ofType: Type, nonNull: Boolean)  extends Type
  }

  final case class FieldResolver(run: Remote[Context] => Remote[DynamicValue])

  implicit val schema: Schema[Document] = DeriveSchema.gen[Document]

  val calibanSchema = new caliban.schema.Schema[DocumentStepGenerator, Document] {
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type = ???

    override def resolve(input: Document): Step[DocumentStepGenerator] =
      Step.QueryStep(ZQuery.fromZIO(DocumentStepGenerator.resolve(input)))
  }
}
