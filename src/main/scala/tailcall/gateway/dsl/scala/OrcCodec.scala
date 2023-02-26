package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.Document
import tailcall.gateway.dsl.scala.Orc._

object OrcCodec {
  def toType(t: Type, isNull: Boolean = true): Document.Type = {
    val nonNull = !isNull
    t match {
      case Type.NonNull(ofType)  => toType(ofType, nonNull)
      case Type.NamedType(name)  => Document.NamedType(name, nonNull)
      case Type.ListType(ofType) => Document.ListType(toType(ofType, nonNull), nonNull)
    }
  }

  def toDefinition(field: LabelledField[Input]): Document.InputValueDefinition =
    Document.InputValueDefinition(field._1, toType(field._2.ofType.getOrElse(???)), field._2.definition.defaultValue)

  def toDefinition(field: LabelledField[Output]): Document.FieldDefinition =
    Document.FieldDefinition(
      name = field._1,
      ofType = toType(field._2.ofType.getOrElse(???)),
      args = field._2.definition.arguments.map(toDefinition),
      resolver = field._2.definition.resolve.getOrElse(???)
    )

  def toDocument(o: Orc): Document = {
    val schemaDefinition = Document
      .SchemaDefinition(query = o.query, mutation = o.mutation, subscription = o.subscription)

    val objectDefinitions: List[Document.Definition] = o.types.collect {
      case Orc.Obj(name, FieldSet.InputSet(fields))  => Document
          .InputObjectTypeDefinition(name, fields.map(toDefinition))
      case Orc.Obj(name, FieldSet.OutputSet(fields)) => Document.ObjectTypeDefinition(name, fields.map(toDefinition))
    }

    Document(schemaDefinition :: objectDefinitions)
  }
}
