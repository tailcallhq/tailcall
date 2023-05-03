package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{
  FieldDefinition,
  InputObjectTypeDefinition,
  InputValueDefinition,
  ObjectTypeDefinition,
}
import caliban.parsing.adt.Type.innerType
import caliban.parsing.adt.{Directive, Document, Type}
import tailcall.runtime.DirectiveCodec.DecoderSyntax
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model._
import zio.json.{DecoderOps, EncoderOps}

/**
 * Used to read a .graphQL file that contains the
 * orchestration specification.
 */
trait Document2Config {

  private val allowedFieldDirectives: List[String] =
    List(InlineType.directive, ModifyField.directive, UnsafeSteps.directive, Http.directive).map(_.name)

  final def toConfig(document: Document): TValid[String, Config] = {
    for {
      schema <- toSchemaDefinition(document)
      types  <- toTypes(document)
      server <- toServer(document)
    } yield Config(server = server, graphQL = Config.GraphQL(schema = schema, types = types))
  }

  final private def assertFieldDirectives(
    typeName: String,
    fieldName: String,
    field: FieldDefinition,
  ): TValid[String, Unit] = {
    TValid.foreach(field.directives)(directive =>
      if (allowedFieldDirectives.contains(directive.name)) TValid.succeed(())
      else TValid.fail(s"${typeName}.${fieldName} has an unrecognized directive: @${directive.name}")
    ).unit

  }

  final private def toField(
    typeName: String,
    fieldName: String,
    field: FieldDefinition,
  ): TValid[String, Config.Field] = {
    for {
      _    <- assertFieldDirectives(typeName, fieldName, field)
      args <- TValid.foreach(field.args)(toLabelledArgument(_)).map(_.toMap)
      steps      = toSteps(field.directives)
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
    } yield Config.Field(
      typeOf = typeof,
      list = Option(isList),
      required = Option(isRequired),
      unsafeSteps = Option(steps),
      args = Option(args),
      doc = field.description,
      modify = field.directives.flatMap(_.fromDirective[ModifyField].toList).headOption,
      http = field.directives.flatMap(_.fromDirective[Http].toList).headOption,
      inline = field.directives.flatMap(_.fromDirective[InlineType].toList).headOption,
    )
  }

  final private def toField(field: InputValueDefinition): TValid[Nothing, Config.Field] =
    TValid.succeed {
      val steps      = toSteps(field.directives)
      val typeof     = innerType(field.ofType)
      val isList     = field.ofType.isInstanceOf[Type.ListType]
      val isRequired = field.ofType.nonNull
      Config.Field(
        typeOf = typeof,
        list = Option(isList),
        required = Option(isRequired),
        unsafeSteps = Option(steps),
        doc = field.description,
        modify = toFieldUpdateAnnotation(field),
      )
    }

  final private def toFieldMap(definition: ObjectTypeDefinition): TValid[String, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(definition.name, field.name, field).map(field.name -> _))
      .map(_.toMap)
  }

  final private def toFieldMap(definition: InputObjectTypeDefinition): TValid[String, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(field).map(field.name -> _)).map(_.toMap)
  }

  private def toFieldUpdateAnnotation(field: InputValueDefinition): Option[ModifyField] = {
    field.directives.flatMap(_.fromDirective[ModifyField].toList).headOption
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

  final private def toSchemaDefinition(document: Document): TValid[String, Config.RootSchema] = {
    document.schemaDefinition match {
      case Some(value) => TValid.succeed(Config.RootSchema(value.query, value.mutation))
      case None        => TValid.succeed(Config.RootSchema())
    }
  }

  final private def toServer(document: Document): TValid[String, Server] = {
    document.schemaDefinition.flatMap(_.directives.find(_.name == "server")) match {
      case Some(directive) => TValid.fromEither(directive.arguments.toJson.fromJson[Server])
      case None            => TValid.succeed(Server())
    }
  }

  private def toSteps(directives: List[Directive]): List[Operation] = {
    TValid.foreach(directives)(_.fromDirective[UnsafeSteps]).toOption.flatMap(_.headOption).toList.flatMap(_.steps)
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

}
