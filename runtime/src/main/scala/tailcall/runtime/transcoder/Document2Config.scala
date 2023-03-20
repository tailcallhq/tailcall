package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{
  FieldDefinition,
  InputValueDefinition,
  ObjectTypeDefinition
}
import caliban.parsing.adt.Type.innerType
import caliban.parsing.adt.{Directive, Document, Type}
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.internal.TValid
import zio.json.{DecoderOps, EncoderOps}

trait Document2Config {
  final def toConfig(document: Document): TValid[String, Config] =
    for {
      schema <- toSchemaDefinition(document)
      types  <- toTypes(document)
      server <- toServer(document)
    } yield Config(server = server, graphQL = Config.GraphQL(schema = schema, types = types))

  final private def toServer(document: Document): TValid[String, Config.Server] = {
    document.schemaDefinition.flatMap(_.directives.find(_.name == "server")) match {
      case Some(directive) => TValid.fromEither(directive.arguments.toJson.fromJson[Config.Server])
      case None            => TValid.succeed(Config.Server())
    }
  }

  final private def toSchemaDefinition(document: Document): TValid[String, Config.SchemaDefinition] = {
    document.schemaDefinition match {
      case Some(value) => TValid.succeed(Config.SchemaDefinition(value.query, value.mutation))
      case None        => TValid.succeed(Config.SchemaDefinition())
    }
  }

  final private def toTypes(document: Document): TValid[String, Map[String, Map[String, Config.Field]]] =
    TValid.foreach(document.objectTypeDefinitions)(definition => toFieldMap(definition).map(definition.name -> _))
      .map(_.toMap)

  final private def toFieldMap(definition: ObjectTypeDefinition): TValid[String, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(toLabelledField(_)).map(_.toMap)
  }

  final private def toLabelledField(field: FieldDefinition): TValid[String, (String, Config.Field)] =
    toField(field).map(field.name -> _)

  final private def toStep(directive: Directive): List[Config.Step] = {
    directive.name match {
      case "steps" => directive.arguments.get("value") match {
          case Some(inputValue) => inputValue.toJson.fromJson[List[Config.Step]].toOption.getOrElse(Nil)
          case None             => Nil
        }
      case _       => Nil
    }
  }

  final private def toField(field: FieldDefinition): TValid[String, Config.Field] =
    for {
      args <- toArgumentMap(field.args)
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
      steps      = field.directives.map(toStep(_)).flatten
    } yield Config.Field(typeof, Option(isList), Option(isRequired), Option(steps), Option(args))

  final private def toArgumentMap(value: List[InputValueDefinition]): TValid[String, Map[String, Config.Argument]] = {
    TValid.foreach(value)(toLabelledArgument(_)).map(_.toMap)
  }

  final private def toLabelledArgument(argument: InputValueDefinition): TValid[String, (String, Config.Argument)] = {
    val typeof     = innerType(argument.ofType)
    val isList     = argument.ofType.isInstanceOf[Type.ListType]
    val isRequired = argument.ofType.nonNull
    TValid.succeed(argument.name, Config.Argument(typeof, Option(isList), Option(isRequired)))
  }

}
