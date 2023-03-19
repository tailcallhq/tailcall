package tailcall.runtime.transcoder

import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.scala.Orc
import tailcall.runtime.dsl.scala.Orc._
import tailcall.runtime.remote._
import tailcall.runtime.transcoder.Transcoder.TExit
import zio.schema.DynamicValue

object Orc2Blueprint {
  def toType(t: Type, isNull: Boolean = true): Blueprint.Type = {
    val nonNull = !isNull
    t match {
      case Type.NonNull(ofType)  => toType(ofType, false)
      case Type.NamedType(name)  => Blueprint.NamedType(name, nonNull)
      case Type.ListType(ofType) => Blueprint.ListType(toType(ofType, isNull), nonNull)
    }
  }

  def toInputValueDefinition(lField: LabelledField[Input]): TExit[String, Blueprint.InputValueDefinition] =
    for {
      ofType <- TExit.fromOption(lField.field.ofType) <> TExit.fail("Input type must be named")
    } yield Blueprint.InputValueDefinition(lField.name, toType(ofType), lField.field.definition.defaultValue)

  def toResolver(lfield: LabelledField[Output]): Option[Remote[DynamicValue] => Remote[DynamicValue]] =
    lfield.field.definition.resolve match {
      case Resolver.Empty           => Option(_.path("value", lfield.name).toDynamic)
      case Resolver.FromFunction(f) => Option(f)
    }
  def toFieldDefinition(lField: LabelledField[Output]): TExit[String, Blueprint.FieldDefinition]      = {
    for {
      ofType <- TExit.fromOption(lField.field.ofType) <> TExit.fail("Output type must be named")
      args   <- TExit.foreach(lField.field.definition.arguments)(toInputValueDefinition)
    } yield Blueprint.FieldDefinition(
      name = lField.name,
      ofType = toType(ofType),
      args = args,
      resolver = toResolver(lField).map(Remote.toLambda(_))
    )
  }

  def toBlueprint(o: Orc): TExit[String, Blueprint] = {
    val schemaDefinition = Blueprint
      .SchemaDefinition(query = o.query, mutation = o.mutation, subscription = o.subscription)

    for {
      objectDefinitions <- TExit.foreach(o.types.map {
        case Orc.Obj(name, FieldSet.InputSet(fields))  => toInputObjectTypeDefinition(name, fields)
        case Orc.Obj(name, FieldSet.OutputSet(fields)) => toObjectTypeDefinition(name, fields)
        case Orc.Obj(name, FieldSet.Empty)             => toObjectTypeDefinition(name, Nil)
      })(identity)
    } yield Blueprint(schemaDefinition :: objectDefinitions)
  }

  private def toObjectTypeDefinition(name: String, fields: List[LabelledField[Output]]) = {
    TExit.foreach(fields)(toFieldDefinition).map(Blueprint.ObjectTypeDefinition(name, _))
  }

  private def toInputObjectTypeDefinition(name: String, fields: List[LabelledField[Input]]) = {
    TExit.foreach(fields)(toInputValueDefinition).map(Blueprint.InputObjectTypeDefinition(name, _))
  }
}
