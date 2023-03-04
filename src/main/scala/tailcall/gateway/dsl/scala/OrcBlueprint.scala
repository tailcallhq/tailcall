package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.Blueprint
import tailcall.gateway.dsl.scala.Orc._
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue
import zio.{IO, ZIO}

object OrcBlueprint {
  def toType(t: Type, isNull: Boolean = true): Blueprint.Type = {
    val nonNull = !isNull
    t match {
      case Type.NonNull(ofType)  => toType(ofType, nonNull)
      case Type.NamedType(name)  => Blueprint.NamedType(name, nonNull)
      case Type.ListType(ofType) => Blueprint.ListType(toType(ofType, nonNull), nonNull)
    }
  }

  def toInputValueDefinition(lField: LabelledField[Input]): IO[String, Blueprint.InputValueDefinition] =
    for {
      ofType <- ZIO.fromOption(lField.field.ofType) <> ZIO.fail("Input type must be named")
    } yield Blueprint.InputValueDefinition(lField.name, toType(ofType), lField.field.definition.defaultValue)

  def toResolver(lfield: LabelledField[Output]): Option[Remote[DynamicValue] => Remote[DynamicValue]] =
    lfield.field.definition.resolve match {
      case Resolver.Empty           => None
      case Resolver.FromFunction(f) => Some(f)
      case Resolver.FromParent      => Some(_.path("value", lfield.name).toDynamic)
    }
  def toFieldDefinition(lField: LabelledField[Output]): IO[String, Blueprint.FieldDefinition]         = {
    for {
      ofType <- ZIO.fromOption(lField.field.ofType) <> ZIO.fail("Output type must be named")
      args   <- ZIO.foreach(lField.field.definition.arguments)(toInputValueDefinition)
    } yield Blueprint
      .FieldDefinition(name = lField.name, ofType = toType(ofType), args = args, resolver = toResolver(lField))
  }

  def toDocument(o: Orc): IO[String, Blueprint] = {
    val schemaDefinition = Blueprint
      .SchemaDefinition(query = o.query, mutation = o.mutation, subscription = o.subscription)

    for {
      objectDefinitions <- ZIO.foreach(o.types.collect {
        case Orc.Obj(name, FieldSet.InputSet(fields))  => toInputObjectTypeDefinition(name, fields)
        case Orc.Obj(name, FieldSet.OutputSet(fields)) => toObjectTypeDefinition(name, fields)
      })(identity)
    } yield Blueprint(schemaDefinition, objectDefinitions)
  }

  private def toObjectTypeDefinition(name: String, fields: List[LabelledField[Output]]) = {
    ZIO.foreach(fields)(toFieldDefinition).map(Blueprint.ObjectTypeDefinition(name, _))
  }

  private def toInputObjectTypeDefinition(name: String, fields: List[LabelledField[Input]]) = {
    ZIO.foreach(fields)(toInputValueDefinition).map(Blueprint.InputObjectTypeDefinition(name, _))
  }
}
