package tailcall.gateway.ast

import tailcall.gateway.remote.Remote
import zio.schema.{DeriveSchema, DynamicValue, Schema}

final case class Document(definition: List[Document.Definition])
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
}
