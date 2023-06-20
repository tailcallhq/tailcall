package tailcall.runtime.transcoder

import caliban.Value
import caliban.parsing.SourceMapper
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{FieldDefinition, InputValueDefinition}
import caliban.parsing.adt.{
  Definition => CalibanDefinition,
  Directive,
  Document => CalibanDocument,
  Type => CalibanType,
}
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Blueprint

/**
 * Converts the blueprint into a the final output document.
 */
trait Blueprint2Document {

  final def toDocument(blueprint: Blueprint): TValid[Nothing, CalibanDocument] =
    TValid.succeed {
      CalibanDocument(
        blueprint.definitions.map {
          case Blueprint.SchemaDefinition(query, mutation, subscription, directives) => CalibanDefinition
              .TypeSystemDefinition
              .SchemaDefinition(directives.map(toCalibanDirective(_)), query, mutation, subscription)
          case Blueprint.ObjectTypeDefinition(name, fields, description, implements) => CalibanDefinition
              .TypeSystemDefinition.TypeDefinition
              .ObjectTypeDefinition(description, name, toCalibanImplements(implements), Nil, fields.map(toCalibanField))
          case Blueprint.InputObjectTypeDefinition(name, fields, description) => CalibanDefinition.TypeSystemDefinition
              .TypeDefinition.InputObjectTypeDefinition(description, name, Nil, fields.map(toCalibanInputValue))
          case Blueprint.ScalarTypeDefinition(name, directives, description)  => CalibanDefinition.TypeSystemDefinition
              .TypeDefinition.ScalarTypeDefinition(description, name, directives.map(toCalibanDirective(_)))
          case Blueprint.InterfaceTypeDefinition(name, fields, description)   => CalibanDefinition.TypeSystemDefinition
              .TypeDefinition.InterfaceTypeDefinition(description, name, Nil, fields.map(toCalibanField))
        },
        SourceMapper.empty,
      )
    }

  final private def toCalibanDirective(directive: Blueprint.Directive): Directive = {
    Directive(
      directive.name,
      directive.arguments.map { case (key, value) => key -> Transcoder.toInputValue(value).getOrElse(Value.NullValue) },
    )
  }

  final private def toCalibanField(field: Blueprint.FieldDefinition): FieldDefinition = {
    val directives = field.directives.map(toCalibanDirective(_))
    FieldDefinition(
      field.description,
      field.name,
      field.args.map(toCalibanInputValue),
      toCalibanType(field.ofType),
      directives,
    )
  }

  final private def toCalibanInputValue(inputValue: Blueprint.InputFieldDefinition): InputValueDefinition =
    CalibanDefinition.TypeSystemDefinition.TypeDefinition.InputValueDefinition(
      inputValue.description,
      inputValue.name,
      toCalibanType(inputValue.ofType),
      inputValue.defaultValue.map(Transcoder.toInputValue(_).getOrElse(Value.NullValue)),
      Nil,
    )

  final private def toCalibanType(tpe: Blueprint.Type): CalibanType =
    tpe match {
      case Blueprint.NamedType(name, nonNull)  => CalibanType.NamedType(name, nonNull)
      case Blueprint.ListType(ofType, nonNull) => CalibanType.ListType(toCalibanType(ofType), nonNull)
    }

  final private def toCalibanImplements(implements: List[Blueprint.NamedType]): List[CalibanType.NamedType] =
    if (implements.nonEmpty) implements.map(t => CalibanType.NamedType(t.name, t.nonNull)) else Nil
}
