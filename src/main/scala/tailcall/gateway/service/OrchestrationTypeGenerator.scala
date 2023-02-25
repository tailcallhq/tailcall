package tailcall.gateway.service

import caliban.introspection.adt.__Type
import caliban.parsing.SourceMapper
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{FieldDefinition, InputValueDefinition}
import caliban.parsing.adt.{Definition => CalibanDefinition, Document => CalibanDocument, Type => CalibanType}
import caliban.tools.RemoteSchema.parseRemoteSchema
import tailcall.gateway.ast.Orchestration
import tailcall.gateway.internal.DynamicValueUtil
import zio.{ZIO, ZLayer}

trait OrchestrationTypeGenerator {
  def __type(doc: Orchestration): __Type
}

object OrchestrationTypeGenerator {
  def __type(document: Orchestration): ZIO[OrchestrationTypeGenerator, Nothing, __Type] =
    ZIO.serviceWith[OrchestrationTypeGenerator](_.__type(document))

  def live: ZLayer[Any, Nothing, OrchestrationTypeGenerator] = ZLayer.succeed(new Live())

  final class Live extends OrchestrationTypeGenerator {
    // TODO: fix this implementation
    override def __type(doc: Orchestration): __Type =
      parseRemoteSchema(toCalibanDocument(doc)).map(_.queryType).getOrElse(???)

    private def toCalibanDocument(document: Orchestration): CalibanDocument = {
      CalibanDocument(
        document.definition.map {
          case Orchestration.ObjectTypeDefinition(name, fields) => CalibanDefinition.TypeSystemDefinition.TypeDefinition
              .ObjectTypeDefinition(None, name, Nil, Nil, fields.map(toCalibanField))
          case Orchestration.InputObjectTypeDefinition(name, fields) => CalibanDefinition.TypeSystemDefinition
              .TypeDefinition.InputObjectTypeDefinition(None, name, Nil, fields.map(toCalibanInputValue))
          case Orchestration.SchemaDefinition(queries, mutations, subscriptions) => CalibanDefinition
              .TypeSystemDefinition.SchemaDefinition(Nil, queries, mutations, subscriptions)
        },
        SourceMapper.empty
      )
    }

    private def toCalibanField(field: Orchestration.FieldDefinition): FieldDefinition =
      FieldDefinition(None, field.name, field.args.map(toCalibanInputValue), toCalibanType(field.ofType), Nil)

    private def toCalibanInputValue(inputValue: Orchestration.InputValueDefinition): InputValueDefinition =
      CalibanDefinition.TypeSystemDefinition.TypeDefinition.InputValueDefinition(
        None,
        inputValue.name,
        toCalibanType(inputValue.ofType),
        inputValue.defaultValue.map(DynamicValueUtil.toInputValue),
        Nil
      )

    private def toCalibanType(tpe: Orchestration.Type): CalibanType =
      tpe match {
        case Orchestration.NamedType(name, nonNull)  => CalibanType.NamedType(name, nonNull)
        case Orchestration.ListType(ofType, nonNull) => CalibanType.ListType(toCalibanType(ofType), nonNull)
      }
  }
}
