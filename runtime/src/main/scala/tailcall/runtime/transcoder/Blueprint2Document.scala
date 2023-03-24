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
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.internal.TValid

trait Blueprint2Document {
  final def toDocument(document: Blueprint): TValid[Nothing, CalibanDocument] =
    TValid.succeed {
      CalibanDocument(
        document.definitions.map {
          case Blueprint.SchemaDefinition(query, mutation, subscription, directives) => CalibanDefinition
              .TypeSystemDefinition
              .SchemaDefinition(directives.map(toCalibanDirective(_)), query, mutation, subscription)
          case Blueprint.ObjectTypeDefinition(name, fields) => CalibanDefinition.TypeSystemDefinition.TypeDefinition
              .ObjectTypeDefinition(None, name, Nil, Nil, fields.map(toCalibanField))
          case Blueprint.InputObjectTypeDefinition(name, fields) => CalibanDefinition.TypeSystemDefinition
              .TypeDefinition.InputObjectTypeDefinition(None, name, Nil, fields.map(toCalibanInputValue))
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
    FieldDefinition(None, field.name, field.args.map(toCalibanInputValue), toCalibanType(field.ofType), directives)
  }

  final private def toCalibanInputValue(inputValue: Blueprint.InputValueDefinition): InputValueDefinition =
    CalibanDefinition.TypeSystemDefinition.TypeDefinition.InputValueDefinition(
      None,
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
}
