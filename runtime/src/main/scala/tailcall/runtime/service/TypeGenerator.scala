package tailcall.runtime.service

import caliban.introspection.adt.__Type
import caliban.parsing.SourceMapper
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{FieldDefinition, InputValueDefinition}
import caliban.parsing.adt.{Definition => CalibanDefinition, Document => CalibanDocument, Type => CalibanType}
import caliban.tools.RemoteSchema.parseRemoteSchema
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.internal.DynamicValueUtil
import zio.{ZIO, ZLayer}

trait TypeGenerator {
  def __type(doc: Blueprint): Option[__Type]
}

object TypeGenerator {
  def __type(document: Blueprint): ZIO[TypeGenerator, Nothing, Option[__Type]] =
    ZIO.serviceWith[TypeGenerator](_.__type(document))

  def live: ZLayer[Any, Nothing, TypeGenerator] = ZLayer.succeed(new Live())

  final class Live extends TypeGenerator {
    // TODO: fix this implementation
    override def __type(doc: Blueprint): Option[__Type] = parseRemoteSchema(toCalibanDocument(doc)).map(_.queryType)

    private def toCalibanDocument(document: Blueprint): CalibanDocument = {
      CalibanDocument(
        CalibanDefinition.TypeSystemDefinition
          .SchemaDefinition(Nil, document.schema.query, document.schema.mutation, document.schema.subscription) ::
          document.definitions.map {
            case Blueprint.ObjectTypeDefinition(name, fields) => CalibanDefinition.TypeSystemDefinition.TypeDefinition
                .ObjectTypeDefinition(None, name, Nil, Nil, fields.map(toCalibanField))
            case Blueprint.InputObjectTypeDefinition(name, fields) => CalibanDefinition.TypeSystemDefinition
                .TypeDefinition.InputObjectTypeDefinition(None, name, Nil, fields.map(toCalibanInputValue))
          },
        SourceMapper.empty
      )
    }

    private def toCalibanField(field: Blueprint.FieldDefinition): FieldDefinition =
      FieldDefinition(None, field.name, field.args.map(toCalibanInputValue), toCalibanType(field.ofType), Nil)

    private def toCalibanInputValue(inputValue: Blueprint.InputValueDefinition): InputValueDefinition =
      CalibanDefinition.TypeSystemDefinition.TypeDefinition.InputValueDefinition(
        None,
        inputValue.name,
        toCalibanType(inputValue.ofType),
        inputValue.defaultValue.map(DynamicValueUtil.toInputValue),
        Nil
      )

    private def toCalibanType(tpe: Blueprint.Type): CalibanType =
      tpe match {
        case Blueprint.NamedType(name, nonNull)  => CalibanType.NamedType(name, nonNull)
        case Blueprint.ListType(ofType, nonNull) => CalibanType.ListType(toCalibanType(ofType), nonNull)
      }
  }
}
