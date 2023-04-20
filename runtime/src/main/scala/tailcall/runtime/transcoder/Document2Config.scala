package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{
  FieldDefinition,
  InputObjectTypeDefinition,
  InputValueDefinition,
  ObjectTypeDefinition,
}
import caliban.parsing.adt.Type.innerType
import caliban.parsing.adt.{Document, Type}
import tailcall.runtime.DirectiveCodec.DecoderSyntax
import tailcall.runtime.internal.TValid
import tailcall.runtime.model._
import zio.json.{DecoderOps, EncoderOps}

/**
 * Used to read a .graphQL file that contains the
 * orchestration specification.
 */
trait Document2Config {

  final def toConfig(document: Document): TValid[String, Config] = {
    for {
      schema <- toSchemaDefinition(document)
      types  <- toTypes(document)
      server <- toServer(document)
    } yield Config(server = server, graphQL = Config.GraphQL(schema = schema, types = types))
  }

  final private def toServer(document: Document): TValid[String, Server] = {
    document.schemaDefinition.flatMap(_.directives.find(_.name == "server")) match {
      case Some(directive) => TValid.fromEither(directive.arguments.toJson.fromJson[Server])
      case None            => TValid.succeed(Server())
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

  final private def toField(field: FieldDefinition): TValid[String, Config.Field] = {
    for {
      args <- TValid.foreach(field.args)(toLabelledArgument(_)).map(_.toMap)
      steps      = TValid.foreach(field.directives)(_.fromDirective[Steps]).map(_.headOption).toOption.flatten
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
    } yield Config.Field(
      typeOf = typeof,
      list = Option(isList),
      required = Option(isRequired),
      steps = steps,
      args = Option(args),
      doc = field.description,
      modify = field.directives.flatMap(_.fromDirective[ModifyField].toList).headOption,
    )
  }

  final private def toLabelledArgument(arg: InputValueDefinition): TValid[String, (String, Config.Arg)] = {
    val typeof     = innerType(arg.ofType)
    val isList     = arg.ofType.isInstanceOf[Type.ListType]
    val isRequired = arg.ofType.nonNull
    TValid.succeed(
      arg.name,
      Config.Arg(
        typeOf = typeof,
        list = Option(isList),
        required = Option(isRequired),
        doc = arg.description,
        modify = toFieldUpdateAnnotation(arg),
      ),
    )
  }

  private def toFieldUpdateAnnotation(field: InputValueDefinition): Option[ModifyField] = {
    field.directives.flatMap(_.fromDirective[ModifyField].toList).headOption
  }

  final private def toFieldMap(definition: InputObjectTypeDefinition): TValid[String, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(field).map(field.name -> _)).map(_.toMap)
  }

  final private def toField(field: InputValueDefinition): TValid[Nothing, Config.Field] =
    TValid.succeed {
      val steps      = TValid.foreach(field.directives)(_.fromDirective[Steps]).map(_.headOption).toOption.flatten
      val typeof     = innerType(field.ofType)
      val isList     = field.ofType.isInstanceOf[Type.ListType]
      val isRequired = field.ofType.nonNull
      Config.Field(
        typeOf = typeof,
        list = Option(isList),
        required = Option(isRequired),
        steps = steps,
        doc = field.description,
        modify = toFieldUpdateAnnotation(field),
      )
    }
}
