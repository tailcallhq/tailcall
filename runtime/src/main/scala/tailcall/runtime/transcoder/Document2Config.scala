package tailcall.runtime.transcoder

import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition._
import caliban.parsing.adt.Type.innerType
import caliban.parsing.adt.{Directive, Document, Type}
import tailcall.runtime.DirectiveCodec.DecoderSyntax
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model._
import zio.json._
import zio.json.ast.Json

/**
 * Used to read a .graphQL file that contains the
 * orchestration specification.
 */
trait Document2Config {
  final def toConfig(document: Document): TValid[Nothing, Config] = {
    for {
      schema <- toSchemaDefinition(document)
      types  <- toTypes(document)
      unions <- toUnions(document)
      server <- toServer(document)
    } yield Config(server = server, graphQL = Config.GraphQL(schema = schema, types = types, unions = Option(unions)))
  }

  final private def toField(field: FieldDefinition): TValid[Nothing, Config.Field] = {
    for {
      args <- TValid.foreach(field.args)(toLabelledArgument(_)).map(_.toMap)
      steps      = toSteps(field.directives)
      typeof     = innerType(field.ofType)
      isList     = field.ofType.isInstanceOf[Type.ListType]
      isRequired = field.ofType.nonNull
    } yield {
      val configField = Config.Field(
        typeOf = typeof,
        list = Option(isList),
        required = Option(isRequired),
        unsafeSteps = Option(steps),
        args = Option(args),
        doc = field.description,
        modify = field.directives.flatMap(_.fromDirective[ModifyField].toList).headOption,
        http = field.directives.flatMap(_.fromDirective[Http].toOption).headOption,
        inline = field.directives.flatMap(_.fromDirective[InlineType].toList).headOption,
      )
      field.directives.flatMap(_.fromDirective[ConstantType].toOption).headOption match {
        case Some(constantValue) => configField
            .resolveWithJson((constantValue.value.fromJson[Json]).getOrElse(Json.Null))
        case None                => configField
      }
    }
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

  final private def toFieldMap(definition: ObjectTypeDefinition): TValid[Nothing, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(field).map(field.name -> _).trace(field.name)).map(_.toMap)
  }

  final private def toFieldMap(definition: InputObjectTypeDefinition): TValid[Nothing, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(field).map(field.name -> _).trace(field.name)).map(_.toMap)
  }

  final private def toFieldMap(definition: InterfaceTypeDefinition): TValid[Nothing, Map[String, Config.Field]] = {
    TValid.foreach(definition.fields)(field => toField(field).map(field.name -> _).trace(field.name)).map(_.toMap)
  }

  private def toFieldUpdateAnnotation(field: InputValueDefinition): Option[ModifyField] = {
    field.directives.flatMap(_.fromDirective[ModifyField].toList).headOption
  }

  final private def toLabelledArgument(arg: InputValueDefinition): TValid[Nothing, (String, Config.Arg)] = {
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

  final private def toSchemaDefinition(document: Document): TValid[Nothing, Config.RootSchema] = {
    document.schemaDefinition match {
      case Some(value) => TValid.succeed(Config.RootSchema(value.query, value.mutation))
      case None        => TValid.succeed(Config.RootSchema())
    }
  }

  final private def toServer(document: Document): TValid[Nothing, Server] =
    TValid.succeed {
      document.schemaDefinition.flatMap(_.directives.find(_.name == Server.directive.name)) match {
        case Some(directive) => Server.directive.decode(directive).getOrElse(Server())
        case None            => Server()
      }
    }

  private def toSteps(directives: List[Directive]): List[Operation] = {
    TValid.foreach(directives)(_.fromDirective[UnsafeSteps]).toOption.flatMap(_.headOption).toList.flatMap(_.steps)
  }

  final private def toTypes(document: Document): TValid[Nothing, Map[String, Config.Type]] = {
    val outputTypes: TValid[Nothing, Map[String, Config.Type]] = TValid
      .foreach(document.objectTypeDefinitions) { definition =>
        toFieldMap(definition).map(fields =>
          definition.name -> Config
            .Type(doc = definition.description, fields = fields, implements = Option(definition.implements.map(_.name)))
        ).trace(definition.name)
      }.map(_.toMap)

    val inputTypes: TValid[Nothing, Map[String, Config.Type]] = TValid
      .foreach(document.inputObjectTypeDefinitions) { definition =>
        toFieldMap(definition)
          .map(fields => definition.name -> Config.Type(doc = definition.description, fields = fields))
          .trace(definition.name)
      }.map(_.toMap)

    val interfaceTypes: TValid[Nothing, Map[String, Config.Type]] = TValid
      .foreach(document.interfaceTypeDefinitions) { definition =>
        toFieldMap(definition)
          .map(fields => definition.name -> Config.Type(doc = definition.description, fields = fields).asInterface)
          .trace(definition.name)
      }.map(_.toMap)

    val enums = TValid.foreach(document.enumTypeDefinitions) { definition =>
      TValid.succeed(
        definition.name -> Config
          .Type(doc = definition.description, variants = Option(definition.enumValuesDefinition.map(_.enumValue)))
      )
    }.map(_.toMap)

    TValid.parWith(outputTypes, inputTypes, interfaceTypes, enums)(_ ++ _)
  }

  final private def toUnions(document: Document): TValid[Nothing, List[Config.Union]] = {
    TValid.foreach(document.unionTypeDefinitions) { definition =>
      TValid.succeed(Config.Union(name = definition.name, doc = definition.description, types = definition.memberTypes))
    }
  }

}
