package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{
  FieldDefinition,
  InputObjectTypeDefinition,
  InputValueDefinition,
  ObjectTypeDefinition,
}
import caliban.parsing.adt.Type.innerType
import caliban.parsing.adt.{Directive, Document, Type}
import tailcall.runtime.http.Method
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.{Config, Path}
import zio.json.{DecoderOps, EncoderOps}

trait Document2Config {
  final def toConfig(document: Document): TValid[String, Config] = {
    for {
      schema <- toSchemaDefinition(document)
      types  <- toTypes(document)
      server <- toServer(document)
    } yield Config(server = server, graphQL = Config.GraphQL(schema = schema, types = types))
  }

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
      toFieldMap(definition).map(definition.name -> Config.Type(doc = definition.description, _))
    }.map(_.toMap)

    val inputTypes = TValid.foreach(document.inputObjectTypeDefinitions) { definition =>
      toFieldMap(definition).map(definition.name -> Config.Type(doc = definition.description, _))
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
      args  <- TValid.foreach(field.args)(toLabelledArgument(_)).map(_.toMap)
      steps <- TValid.foreach(field.directives)(toStep(_)).map(_.flatten)
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
    } yield Config.Field(
      typeOf = typeof,
      list = Option(isList),
      required = Option(isRequired),
      steps = Option(steps),
      args = Option(args),
      doc = field.description,
    )

  final private def toField(field: InputValueDefinition): TValid[String, Config.Field] =
    for {
      steps <- TValid.foreach(field.directives)(toStep(_)).map(_.flatten)
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
    } yield Config.Field(
      typeOf = typeof,
      list = Option(isList),
      required = Option(isRequired),
      steps = Option(steps),
      doc = field.description,
    )

  final private def toLabelledArgument(arg: InputValueDefinition): TValid[String, (String, Config.Arg)] = {
    val typeof     = innerType(arg.ofType)
    val isList     = arg.ofType.isInstanceOf[Type.ListType]
    val isRequired = arg.ofType.nonNull
    TValid.succeed(
      arg.name,
      Config.Arg(typeOf = typeof, list = Option(isList), required = Option(isRequired), doc = arg.description),
    )
  }

}
