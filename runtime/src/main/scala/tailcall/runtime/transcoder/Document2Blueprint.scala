package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.InputValueDefinition
import caliban.parsing.adt.{Definition, Document, Type}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.transcoder.Transcoder.Syntax
import zio.schema.DynamicValue

object Document2Blueprint extends Transcoder[Document, String, Blueprint] {
  private def toBlueprintType(tpe: Type): Blueprint.Type = {
    tpe match {
      case Type.NamedType(name, nonNull)  => Blueprint.NamedType(name, nonNull)
      case Type.ListType(ofType, nonNull) => Blueprint.ListType(toBlueprintType(ofType), nonNull)
    }
  }

  private def toBlueprintInputValueDefinition(
    inputValueDefinition: InputValueDefinition
  ): TExit[String, Blueprint.InputValueDefinition] =
    inputValueDefinition.defaultValue
      .fold[TExit[String, Option[DynamicValue]]](TExit.none)(_.transcode[DynamicValue, String].some)
      .map(Blueprint.InputValueDefinition(inputValueDefinition.name, toBlueprintType(inputValueDefinition.ofType), _))

  private def toBlueprintFieldDefinition(
    fieldDefinition: Definition.TypeSystemDefinition.TypeDefinition.FieldDefinition
  ): TExit[String, Blueprint.FieldDefinition] =
    TExit.foreach(fieldDefinition.args)(toBlueprintInputValueDefinition(_))
      .map(args => Blueprint.FieldDefinition(fieldDefinition.name, args, toBlueprintType(fieldDefinition.ofType)))

  private def toBlueprintDefinition(definition: Definition): TExit[String, Option[Blueprint.Definition]] = {
    definition match {
      case _: Definition.ExecutableDefinition          => TExit.fail("Executable definitions are not supported yet")
      case definition: Definition.TypeSystemDefinition => definition match {
          case TypeSystemDefinition.SchemaDefinition(_, query, mutation, subscription) => TExit
              .succeed(Option(Blueprint.SchemaDefinition(query, mutation, subscription)))
          case _: TypeSystemDefinition.DirectiveDefinition => TExit.fail("Directive definitions are not supported yet")
          case definition: TypeSystemDefinition.TypeDefinition => definition match {
              case TypeDefinition.ObjectTypeDefinition(_, name, _, _, fields) => TExit
                  .foreach(fields)(toBlueprintFieldDefinition(_))
                  .map(fields => Option(Blueprint.ObjectTypeDefinition(name, fields)))
              case _: TypeDefinition.InterfaceTypeDefinition => TExit.fail("Interface types are not supported yet")
              case TypeDefinition.InputObjectTypeDefinition(_, name, _, fields) => TExit
                  .foreach(fields)(toBlueprintInputValueDefinition(_))
                  .map(fields => Option(Blueprint.InputObjectTypeDefinition(name, fields)))
              case _: TypeDefinition.EnumTypeDefinition   => TExit.fail("Enum types are not supported yet")
              case _: TypeDefinition.UnionTypeDefinition  => TExit.fail("Union types are not supported yet")
              case _: TypeDefinition.ScalarTypeDefinition => TExit.fail("Scalar types are not supported yet")
            }
        }
      case _: Definition.TypeSystemExtension           => TExit.fail("Type system extensions are not supported yet")
    }
  }

  override def run(document: Document): TExit[String, Blueprint] = {
    TExit.foreach(document.definitions)(toBlueprintDefinition(_))
      .map(defs => Blueprint(defs.collect { case Some(d) => d }))
  }
}
