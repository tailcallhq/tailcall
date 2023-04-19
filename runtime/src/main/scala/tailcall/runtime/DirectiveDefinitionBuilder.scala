package tailcall.runtime

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.InputValueDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.{DirectiveDefinition, DirectiveLocation}
import tailcall.runtime.internal.TValid
import tailcall.runtime.transcoder.Transcoder
import zio.Chunk
import zio.schema.annotation.caseName
import zio.schema.meta.ExtensibleMetaSchema
import zio.schema.{Schema, StandardType}

final case class DirectiveDefinitionBuilder[A](
  schema: Schema[A],
  locations: Set[DirectiveLocation] = Set.empty,
  description: Option[String] = None,
) {
  def unsupported(name: String): TValid[String, Nothing] =
    TValid.fail(s"""Can not convert "$name" to caliban "InputValueDefinition"""")

  def unsafeBuild: DirectiveDefinition = build.getOrElse(error => throw new RuntimeException(error))

  def build: TValid[String, DirectiveDefinition] = {
    val maybeName = schema.annotations.collectFirst { case caseName(name) => name }
    schema.ast match {
      case ExtensibleMetaSchema.Product(id, _, fields, _) => for {
          args <- TValid.foreachChunk(fields) { field =>
            for {
              ofType <- Transcoder.toCalibanType(field.schema)
            } yield InputValueDefinition(
              name = field.label,
              description = None,
              ofType = ofType,
              defaultValue = None,
              directives = Nil,
            )
          }
        } yield DirectiveDefinition(description, maybeName.getOrElse(id.name), args.toList, locations)

      case ExtensibleMetaSchema.Value(valueType, _, optional) => for {
          args <-
            if (valueType == StandardType.UnitType) TValid.succeed(Chunk.empty)
            else Transcoder.toCalibanType(valueType, optional).map { ofType =>
              Chunk.single(InputValueDefinition(
                name = valueType.tag.capitalize,
                description = None,
                ofType = ofType,
                defaultValue = None,
                directives = Nil,
              ))
            }
        } yield DirectiveDefinition(description, valueType.tag.capitalize, args.toList, locations)
      case ExtensibleMetaSchema.Tuple(_, _, _, _)             => unsupported("Tuple")
      case ExtensibleMetaSchema.Sum(_, _, _, _)               => unsupported("Sum")
      case ExtensibleMetaSchema.Either(_, _, _, _)            => unsupported("Either")
      case ExtensibleMetaSchema.FailNode(_, _, _)             => unsupported("FailNode")
      case ExtensibleMetaSchema.ListNode(_, _, _)             => unsupported("ListNode")
      case ExtensibleMetaSchema.Dictionary(_, _, _, _)        => unsupported("Dictionary")
      case ExtensibleMetaSchema.Ref(_, _, _)                  => unsupported("Ref")
      case ExtensibleMetaSchema.Known(_, _, _)                => unsupported("Known")
    }
  }

  def withLocations(locations: DirectiveLocation*): DirectiveDefinitionBuilder[A] = copy(locations = locations.toSet)
  def withDescription(description: String): DirectiveDefinitionBuilder[A] = copy(description = Option(description))
  def withType[B: Schema]: DirectiveDefinitionBuilder[B]                  = copy(schema = Schema[B])
}

object DirectiveDefinitionBuilder {
  def make[A: Schema]: DirectiveDefinitionBuilder[A] = DirectiveDefinitionBuilder(Schema[A])
}
