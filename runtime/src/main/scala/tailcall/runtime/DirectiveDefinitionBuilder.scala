package tailcall.runtime

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.InputValueDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.{DirectiveDefinition, DirectiveLocation}
import tailcall.runtime.internal.TValid
import tailcall.runtime.transcoder.Transcoder
import zio.schema.Schema
import zio.schema.annotation.caseName

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
    schema match {
      case schema: Schema.Record[_] => for {
          args <- TValid.foreachChunk(schema.fields) { field =>
            for {
              ofType <- Transcoder.toCalibanType(field.schema, nonNull = true)
            } yield InputValueDefinition(
              name = field.name,
              description = None,
              ofType = ofType,
              defaultValue = None,
              directives = Nil,
            )
          }
        } yield DirectiveDefinition(description, maybeName.getOrElse(schema.id.name), args.toList, locations)
      case _                        => TValid.fail("Can create directive definition only from record type")
    }
  }

  def withLocations(locations: DirectiveLocation*): DirectiveDefinitionBuilder[A] = copy(locations = locations.toSet)
  def withDescription(description: String): DirectiveDefinitionBuilder[A] = copy(description = Option(description))
  def withType[B: Schema]: DirectiveDefinitionBuilder[B]                  = copy(schema = Schema[B])
}

object DirectiveDefinitionBuilder {
  def make[A: Schema]: DirectiveDefinitionBuilder[A] = DirectiveDefinitionBuilder(Schema[A])
}
