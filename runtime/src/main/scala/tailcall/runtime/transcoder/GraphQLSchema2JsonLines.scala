package tailcall.runtime.transcoder

import caliban.CalibanError.ParsingError
import caliban.parsing.adt.Definition
import caliban.parsing.{Parser, adt}
import tailcall.runtime.model.Blueprint
import tailcall.runtime.transcoder.GraphQLSchema2JsonLines.TrainingRow
import zio.ZIO
import zio.json.{DeriveJsonCodec, EncoderOps, JsonCodec}

trait GraphQLSchema2JsonLines {
  def toJsonLines(schema: String): ZIO[Any, ParsingError, String] = {
    for {
      document <- Parser.parseQuery(schema)
      supportedDocument = document.copy(definitions = document.definitions.filter(isSupported))
      blueprint <- Transcoder.toBlueprint(supportedDocument).toZIO
        .catchAll(errors => ZIO.fail(ParsingError(errors.mkString(", "))))
    } yield toTrainingRows(blueprint.definitions).map(_.toJson).mkString("\n")
  }

  private def isSupported(definition: Definition): Boolean =
    definition match {
      case _: adt.Definition.TypeSystemDefinition.TypeDefinition.ObjectTypeDefinition      => true
      case _: adt.Definition.TypeSystemDefinition.TypeDefinition.InputObjectTypeDefinition => true
      case _                                                                               => false
    }

  private def toTrainingRows(definitions: List[Blueprint.Definition]): List[TrainingRow] = {
    definitions.collect { case d: Blueprint.ObjectTypeDefinition =>
      TrainingRow(prompt = d.fields.map(f => f.name -> f.ofType.defaultName).toMap, completion = d.name)
    }
  }
}

object GraphQLSchema2JsonLines {
  final case class TrainingRow(prompt: Map[String, String], completion: String)
  implicit val jsonCodec: JsonCodec[TrainingRow] = DeriveJsonCodec.gen[TrainingRow]
}
