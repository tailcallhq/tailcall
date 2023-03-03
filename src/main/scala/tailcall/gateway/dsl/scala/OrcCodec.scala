package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.Document
import tailcall.gateway.ast.Document.Resolver
import tailcall.gateway.dsl.scala.Orc._
import zio.{IO, ZIO}

object OrcCodec {
  def toType(t: Type, isNull: Boolean = true): Document.Type = {
    val nonNull = !isNull
    t match {
      case Type.NonNull(ofType)  => toType(ofType, nonNull)
      case Type.NamedType(name)  => Document.NamedType(name, nonNull)
      case Type.ListType(ofType) => Document.ListType(toType(ofType, nonNull), nonNull)
    }
  }

  def toInputValueDefinition(lField: LabelledField[Input]): IO[String, Document.InputValueDefinition] =
    for {
      ofType <- ZIO.fromOption(lField.field.ofType) <> ZIO.fail("Input type must be named")
    } yield Document.InputValueDefinition(lField.name, toType(ofType), lField.field.definition.defaultValue)

  def toFieldDefinition(lField: LabelledField[Output]): IO[String, Document.FieldDefinition] = {
    for {
      ofType   <- ZIO.fromOption(lField.field.ofType) <> ZIO.fail("Output type must be named")
      resolver <- ZIO.fromOption(lField.field.definition.resolve) <> ZIO.fail("Output must have a resolver")
      args     <- ZIO.foreach(lField.field.definition.arguments)(toInputValueDefinition)
    } yield Document.FieldDefinition(
      name = lField.name,
      ofType = toType(ofType),
      args = args,
      resolver = Resolver.fromFunction(resolver)
    )
  }

  def toDocument(o: Orc): IO[String, Document] = {
    val schemaDefinition = Document
      .SchemaDefinition(query = o.query, mutation = o.mutation, subscription = o.subscription)

    for {
      objectDefinitions <- ZIO.foreach(o.types.collect {
        case Orc.Obj(name, FieldSet.InputSet(fields))  => toInputObjectTypeDefinition(name, fields)
        case Orc.Obj(name, FieldSet.OutputSet(fields)) => toObjectTypeDefinition(name, fields)
      })(identity)
    } yield Document(schemaDefinition :: objectDefinitions)
  }

  private def toObjectTypeDefinition(name: String, fields: List[LabelledField[Output]]) = {
    ZIO.foreach(fields)(toFieldDefinition).map(Document.ObjectTypeDefinition(name, _))
  }

  private def toInputObjectTypeDefinition(name: String, fields: List[LabelledField[Input]]) = {
    ZIO.foreach(fields)(toInputValueDefinition).map(Document.InputObjectTypeDefinition(name, _))
  }
}
