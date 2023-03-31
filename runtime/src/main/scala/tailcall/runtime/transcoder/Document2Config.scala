package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{
  FieldDefinition,
  InputObjectTypeDefinition,
  InputValueDefinition,
  ObjectTypeDefinition,
}
import caliban.parsing.adt.Type.innerType
import caliban.parsing.adt.{Directive, Document, Type}
import tailcall.runtime.ast.Path
import tailcall.runtime.dsl.Config
import tailcall.runtime.http.Method
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

  final private def toSchemaDefinition(document: Document): TValid[String, Config.RootSchema] = {
    document.schemaDefinition match {
      case Some(value) => TValid.succeed(Config.RootSchema(value.query, value.mutation))
      case None        => TValid.succeed(Config.RootSchema())
    }
  }

  final private def toTypes(document: Document): TValid[String, Map[String, Config.Type]] = {
    val outputTypes = TValid.foreach(document.objectTypeDefinitions) { definition =>
      toFieldMap(definition).map(definition.name -> Config.Type(definition.description, _))
    }.map(_.toMap)

    val inputTypes = TValid.foreach(document.inputObjectTypeDefinitions) { definition =>
      toFieldMap(definition).map(definition.name -> Config.Type(definition.description, _))
    }.map(_.toMap)

    (outputTypes zip inputTypes)(_ ++ _)
  }

  final private def toFieldMap(definition: ObjectTypeDefinition): TValid[String, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(field).map(field.name -> _)).map(_.toMap)
  }

  final private def toFieldMap(definition: InputObjectTypeDefinition): TValid[String, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(field).map(field.name -> _)).map(_.toMap)
  }

  final private def toStep(directive: Directive): TValid[String, List[Config.Step]] = {
    directive.name match {
      case "steps" => directive.arguments.get("value") match {
          case Some(inputValue) => TValid.fromEither(inputValue.toJson.fromJson[List[Config.Step]])
          case None             => TValid.succeed(Nil)
        }

      case "http" => for {
          method <- TValid.fromEither(directive.arguments.get("method").toJson.fromJson[Option[Method]])
          path   <- directive.arguments.get("path") match {
            case None        => TValid.fail("Missing url in @http directive")
            case Some(value) => TValid.fromEither(value.toJson.fromJson[String].flatMap(Path.decode(_)))
          }
        } yield List(Config.Step.Http(method = method, path = path))
      case _      => TValid.succeed(Nil)
    }
  }

  final private def toField(field: FieldDefinition): TValid[String, Config.Field] =
    for {
      args  <- toArgumentMap(field.args)
      steps <- TValid.foreach(field.directives)(toStep(_)).map(_.flatten)
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
    } yield Config.Field(typeof, Option(isList), Option(isRequired), Option(steps), Option(args))

  final private def toField(field: InputValueDefinition): TValid[String, Config.Field] =
    for {
      steps <- TValid.foreach(field.directives)(toStep(_)).map(_.flatten)
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
    } yield Config.Field(typeof, Option(isList), Option(isRequired), Option(steps))

  final private def toArgumentMap(value: List[InputValueDefinition]): TValid[String, Map[String, Config.Arg]] = {
    TValid.foreach(value)(toLabelledArgument(_)).map(_.toMap)
  }

  final private def toLabelledArgument(argument: InputValueDefinition): TValid[String, (String, Config.Arg)] = {
    val typeof     = innerType(argument.ofType)
    val isList     = argument.ofType.isInstanceOf[Type.ListType]
    val isRequired = argument.ofType.nonNull
    TValid.succeed(argument.name, Config.Arg(typeof, Option(isList), Option(isRequired)))
  }

}
