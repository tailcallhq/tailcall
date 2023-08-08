package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.InputValueDefinition
import caliban.parsing.adt.{Definition, Document, Type}
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Blueprint
import zio.schema.DynamicValue

trait Document2Blueprint {
  final private def toBlueprintType(tpe: Type): Blueprint.Type = {
    tpe match {
      case Type.NamedType(name, nonNull)  => Blueprint.NamedType(name, nonNull)
      case Type.ListType(ofType, nonNull) => Blueprint.ListType(toBlueprintType(ofType), nonNull)
    }
  }

  final private def toBlueprintInputValueDefinition(
    inputValueDefinition: InputValueDefinition
  ): TValid[String, Blueprint.InputFieldDefinition] =
    inputValueDefinition.defaultValue
      .fold[TValid[String, Option[DynamicValue]]](TValid.none)(Transcoder.toDynamicValue(_).some)
      .map(Blueprint.InputFieldDefinition(inputValueDefinition.name, toBlueprintType(inputValueDefinition.ofType), _))

  final private def toBlueprintFieldDefinition(
    fieldDefinition: Definition.TypeSystemDefinition.TypeDefinition.FieldDefinition
  ): TValid[String, Blueprint.FieldDefinition] = {
    TValid.foreach(fieldDefinition.args)(toBlueprintInputValueDefinition(_)).map(args =>
      Blueprint
        .FieldDefinition(name = fieldDefinition.name, args = args, ofType = toBlueprintType(fieldDefinition.ofType))
    )
  }

  final def toBlueprintDefinition(definition: Definition): TValid[String, Option[Blueprint.Definition]] = {
    definition match {
      case _: Definition.ExecutableDefinition          => TValid.fail("Executable definitions are not supported yet")
      case definition: Definition.TypeSystemDefinition => definition match {
          case TypeSystemDefinition.SchemaDefinition(_, _, _, _) => TValid.succeed(None)
          case _: TypeSystemDefinition.DirectiveDefinition => TValid.fail("Directive definitions are not supported yet")
          case definition: TypeSystemDefinition.TypeDefinition => definition match {
              case TypeDefinition.ObjectTypeDefinition(_, name, _, _, fields) => TValid
                  .foreach(fields)(toBlueprintFieldDefinition(_))
                  .map(fields => Option(Blueprint.ObjectTypeDefinition(name, fields)))
              case _: TypeDefinition.InterfaceTypeDefinition => TValid.fail("Interface types are not supported yet")
              case TypeDefinition.InputObjectTypeDefinition(_, name, _, fields) => TValid
                  .foreach(fields)(toBlueprintInputValueDefinition(_))
                  .map(fields => Option(Blueprint.InputObjectTypeDefinition(name, fields)))
              case _: TypeDefinition.EnumTypeDefinition            => TValid.fail("Enum types are not supported yet")
              case _: TypeDefinition.UnionTypeDefinition           => TValid.fail("Union types are not supported yet")
              case definition: TypeDefinition.ScalarTypeDefinition => TValid
                  .some(Blueprint.ScalarTypeDefinition(definition.name, Nil, definition.description))
            }
        }
      case _: Definition.TypeSystemExtension           => TValid.fail("Type system extensions are not supported yet")
    }
  }
  final private def toSchemaDefinition(definition: Definition): Option[Blueprint.SchemaDefinition]      =
    definition match {
      case definition: TypeSystemDefinition => definition match {
          case TypeSystemDefinition.SchemaDefinition(_, query, mutation, subscription) =>
            Option(Blueprint.SchemaDefinition(query, mutation, subscription))
          case _                                                                       => None
        }
      case _                                => None
    }

  final def toBlueprint(document: Document): TValid[String, Blueprint] = {
    val schemaDefinition =
      document.definitions.collectFirst { case d: TypeSystemDefinition => d }.flatMap(toSchemaDefinition(_)) match {
        case Some(value) => TValid.succeed(value)
        case None        => TValid.fail("Schema definition is missing")
      }
    schemaDefinition.flatMap(sd =>
      TValid.foreach(document.definitions)(toBlueprintDefinition(_))
        .map(defs => Blueprint(defs.collect { case Some(d) => d }, sd))
    )

  }
}
