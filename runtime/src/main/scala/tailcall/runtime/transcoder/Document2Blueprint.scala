package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.InputValueDefinition
import caliban.parsing.adt.{Definition, Document, Type}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.transcoder.Transcoder.Syntax
import zio.schema.DynamicValue

trait Document2Blueprint {
  private def toBlueprintType(tpe: Type): Blueprint.Type = {
    tpe match {
      case Type.NamedType(name, nonNull)  => Blueprint.NamedType(name, nonNull)
      case Type.ListType(ofType, nonNull) => Blueprint.ListType(toBlueprintType(ofType), nonNull)
    }
  }

  private def toBlueprintInputValueDefinition(
    inputValueDefinition: InputValueDefinition
  ): TValid[String, Blueprint.InputValueDefinition] =
    inputValueDefinition.defaultValue
      .fold[TValid[String, Option[DynamicValue]]](TValid.none)(_.transcode[DynamicValue, String].some)
      .map(Blueprint.InputValueDefinition(inputValueDefinition.name, toBlueprintType(inputValueDefinition.ofType), _))

  private def toBlueprintFieldDefinition(
    fieldDefinition: Definition.TypeSystemDefinition.TypeDefinition.FieldDefinition
  ): TValid[String, Blueprint.FieldDefinition] =
    TValid.foreach(fieldDefinition.args)(toBlueprintInputValueDefinition(_))
      .map(args => Blueprint.FieldDefinition(fieldDefinition.name, args, toBlueprintType(fieldDefinition.ofType)))

  private def toBlueprintDefinition(definition: Definition): TValid[String, Option[Blueprint.Definition]] = {
    definition match {
      case _: Definition.ExecutableDefinition          => TValid.fail("Executable definitions are not supported yet")
      case definition: Definition.TypeSystemDefinition => definition match {
          case TypeSystemDefinition.SchemaDefinition(_, query, mutation, subscription) => TValid
              .succeed(Option(Blueprint.SchemaDefinition(query, mutation, subscription)))
          case _: TypeSystemDefinition.DirectiveDefinition => TValid.fail("Directive definitions are not supported yet")
          case definition: TypeSystemDefinition.TypeDefinition => definition match {
              case TypeDefinition.ObjectTypeDefinition(_, name, _, _, fields) => TValid
                  .foreach(fields)(toBlueprintFieldDefinition(_))
                  .map(fields => Option(Blueprint.ObjectTypeDefinition(name, fields)))
              case _: TypeDefinition.InterfaceTypeDefinition => TValid.fail("Interface types are not supported yet")
              case TypeDefinition.InputObjectTypeDefinition(_, name, _, fields) => TValid
                  .foreach(fields)(toBlueprintInputValueDefinition(_))
                  .map(fields => Option(Blueprint.InputObjectTypeDefinition(name, fields)))
              case _: TypeDefinition.EnumTypeDefinition   => TValid.fail("Enum types are not supported yet")
              case _: TypeDefinition.UnionTypeDefinition  => TValid.fail("Union types are not supported yet")
              case _: TypeDefinition.ScalarTypeDefinition => TValid.fail("Scalar types are not supported yet")
            }
        }
      case _: Definition.TypeSystemExtension           => TValid.fail("Type system extensions are not supported yet")
    }
  }

  final def toBlueprint(document: Document): TValid[String, Blueprint] =
    TValid.foreach(document.definitions)(toBlueprintDefinition(_))
      .map(defs => Blueprint(defs.collect { case Some(d) => d }))
}
