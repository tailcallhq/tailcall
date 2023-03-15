package tailcall.runtime.service

import caliban.introspection.adt.__Schema
import caliban.parsing.SourceMapper
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{FieldDefinition, InputValueDefinition}
import caliban.parsing.adt.{Definition => CalibanDefinition, Document => CalibanDocument, Type => CalibanType}
import caliban.tools.RemoteSchema.parseRemoteSchema
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.internal.DynamicValueUtil
import zio.{ZIO, ZLayer}

trait SchemaGenerator {
  def __schema(doc: Blueprint): Option[__Schema]
}

object SchemaGenerator {

  def __schema(document: Blueprint): ZIO[SchemaGenerator, Nothing, Option[__Schema]] =
    ZIO.serviceWith[SchemaGenerator](_.__schema(document))

  def live: ZLayer[Any, Nothing, SchemaGenerator] = ZLayer.succeed(new Live())

  final class Live extends SchemaGenerator {
    override def __schema(doc: Blueprint): Option[__Schema] = parseRemoteSchema(toCalibanDocument(doc))

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
        inputValue.defaultValue.map(DynamicValueUtil.toInputValue(_).get),
        Nil
      )

    private def toCalibanType(tpe: Blueprint.Type): CalibanType =
      tpe match {
        case Blueprint.NamedType(name, nonNull)  => CalibanType.NamedType(name, nonNull)
        case Blueprint.ListType(ofType, nonNull) => CalibanType.ListType(toCalibanType(ofType), nonNull)
      }
  }
}
