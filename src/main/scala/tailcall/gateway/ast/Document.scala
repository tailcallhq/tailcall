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
}

object Document {
  sealed trait Definition
  object Definition {

    case class ObjectTypeDefinition(name: String, fields: List[FieldDefinition])                    extends Definition
    case class InputObjectTypeDefinition(name: String, fields: List[InputValueDefinition])          extends Definition
    case class InputValueDefinition(name: String, ofType: Type, defaultValue: Option[DynamicValue]) extends Definition
    case class FieldDefinition(name: String, args: List[InputValueDefinition], ofType: Type, resolver: FieldResolver)
        extends Definition
  }

  sealed trait Type
  object Type {
    case class NamedType(name: String, nonNull: Boolean) extends Type
    case class ListType(ofType: Type, nonNull: Boolean)  extends Type
  }

  sealed trait FieldResolver
  object FieldResolver {
    case object Identity                                                     extends FieldResolver
    final case class FromContext(f: Remote[Context] => Remote[DynamicValue]) extends FieldResolver
  }

  implicit val schema: Schema[Document] = DeriveSchema.gen[Document]

  val calibanSchema = new caliban.schema.Schema[DocumentStepGenerator, Document] {
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type = ???

    override def resolve(input: Document): Step[DocumentStepGenerator] =
      Step.QueryStep(ZQuery.fromZIO(DocumentStepGenerator.resolve(input)))
  }
}
