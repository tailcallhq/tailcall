package tailcall.runtime.transcoder

import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Orc._
import tailcall.runtime.model.{Blueprint, Orc}
import tailcall.runtime.remote._
import zio.schema.DynamicValue

trait Orc2Blueprint {
  final def toType(t: Type, isNull: Boolean = true): Blueprint.Type = {
    val nonNull = !isNull
    t match {
      case Type.NonNull(ofType)  => toType(ofType, false)
      case Type.NamedType(name)  => Blueprint.NamedType(name, nonNull)
      case Type.ListType(ofType) => Blueprint.ListType(toType(ofType, isNull), nonNull)
    }
  }

  final private def toInputValueDefinition(
    lField: LabelledField[Input]
  ): TValid[String, Blueprint.InputFieldDefinition] =
    for {
      ofType <- TValid.fromOption(lField.field.ofType) <> TValid.fail("Input type must be named")
    } yield Blueprint.InputFieldDefinition(lField.name, toType(ofType), lField.field.definition.defaultValue)

  final private def toResolver(lfield: LabelledField[Output]): Option[Remote[DynamicValue] => Remote[DynamicValue]] =
    lfield.field.definition.resolve match {
      case Resolver.Empty           => Option(_.path("value", lfield.name).toDynamic)
      case Resolver.FromFunction(f) => Option(f)
    }
  final private def toFieldDefinition(lField: LabelledField[Output]): TValid[String, Blueprint.FieldDefinition]     = {
    for {
      ofType <- TValid.fromOption(lField.field.ofType) <> TValid.fail("Output type must be named")
      args   <- TValid.foreach(lField.field.definition.arguments)(toInputValueDefinition)
    } yield Blueprint.FieldDefinition(
      name = lField.name,
      ofType = toType(ofType),
      args = args,
      resolver = toResolver(lField).map(Remote.toLambda(_)),
    )
  }

  final def toBlueprint(o: Orc): TValid[String, Blueprint] = {
    val schemaDefinition = Blueprint
      .SchemaDefinition(query = o.query, mutation = o.mutation, subscription = o.subscription)

    for {
      objectDefinitions <- TValid.foreach(o.types.map {
        case Orc.Obj(name, FieldSet.InputSet(fields))  => toInputObjectTypeDefinition(name, fields)
        case Orc.Obj(name, FieldSet.OutputSet(fields)) => toObjectTypeDefinition(name, fields)
        case Orc.Obj(name, FieldSet.Empty)             => toObjectTypeDefinition(name, Nil)
      })(identity)
    } yield Blueprint(schemaDefinition :: objectDefinitions)
  }

  final private def toObjectTypeDefinition(name: String, fields: List[LabelledField[Output]]) = {
    TValid.foreach(fields)(toFieldDefinition).map(Blueprint.ObjectTypeDefinition(name, _))
  }

  final private def toInputObjectTypeDefinition(name: String, fields: List[LabelledField[Input]]) = {
    TValid.foreach(fields)(toInputValueDefinition).map(Blueprint.InputObjectTypeDefinition(name, _))
  }

}
