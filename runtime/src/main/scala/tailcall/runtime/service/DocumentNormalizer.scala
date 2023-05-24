package tailcall.runtime.service

import caliban.parsing.adt.Definition.TypeSystemDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition
import caliban.parsing.adt.{Definition, Document}

object DocumentNormalizer {

  /**
   * Normalizes the given document by ensuring all fields
   * and types are sorted in alphabetical order.
   */
  def normalize(document: Document): Document = {
    document.copy(definitions = document.definitions.map {
      case definition: TypeDefinition.ObjectTypeDefinition => definition.copy(fields = definition.fields.sortBy(_.name))
      case definition: TypeDefinition.InputObjectTypeDefinition => definition
          .copy(fields = definition.fields.sortBy(_.name))
      case definition                                           => definition
    }.sortBy[String] {
      case _: Definition.ExecutableDefinition          => ""
      case _: Definition.TypeSystemExtension           => ""
      case definition: Definition.TypeSystemDefinition => definition match {
          case _: TypeSystemDefinition.DirectiveDefinition     => "a"
          case _: TypeSystemDefinition.SchemaDefinition        => "b"
          case definition: TypeSystemDefinition.TypeDefinition => definition match {
              case _: TypeDefinition.ScalarTypeDefinition      => "c" + definition.name
              case _: TypeDefinition.InputObjectTypeDefinition => "d" + definition.name
              case _: TypeDefinition.ObjectTypeDefinition      => "e" + definition.name
              case _: TypeDefinition.InterfaceTypeDefinition   => definition.name
              case _: TypeDefinition.EnumTypeDefinition        => definition.name
              case _: TypeDefinition.UnionTypeDefinition       => definition.name
            }
        }
    })
  }
}
