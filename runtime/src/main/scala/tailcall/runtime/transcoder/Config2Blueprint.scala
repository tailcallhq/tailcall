package tailcall.runtime.transcoder

import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.internal.TValid
import tailcall.runtime.lambda.Syntax._
import tailcall.runtime.lambda._
import tailcall.runtime.model.Blueprint.NamedType
import tailcall.runtime.model.Config._
import tailcall.runtime.model.Mustache.MustacheExpression
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model._
import zio.Chunk
import zio.schema.DynamicValue

trait Config2Blueprint {
  def toBlueprint(config: Config): TValid[String, Blueprint] = Config2Blueprint.Live(config).toBlueprint
}

object Config2Blueprint {
  final case class Live(config: Config) {
    private val outputTypes: Set[String] = config.outputTypes.toSet
    private val inputTypes: Set[String]  = config.inputTypes.toSet

    /**
     * Encodes a config into a Blueprint.
     */
    def toBlueprint: TValid[String, Blueprint] = {
      val rootSchema = Blueprint
        .SchemaDefinition(query = config.graphQL.schema.query, mutation = config.graphQL.schema.mutation)

      for { definitions <- toDefinitions } yield Blueprint(rootSchema :: definitions)
    }

    private def appendResolver(
      bField: Blueprint.FieldDefinition,
      f: DynamicValue ~> DynamicValue,
    ): Blueprint.FieldDefinition =
      bField.resolver match {
        case Some(g) => bField.copy(resolver = Option(g >>> f))
        case None    => bField.copy(resolver = Option(f))
      }

    private def assertTypeName(typeName: String, isInput: Boolean): TValid[String, Unit] = {
      val inInputs  = inputTypes.contains(typeName)
      val inOutputs = outputTypes.contains(typeName)
      TValid.fail(s"undefined input name $typeName").when(isInput && !inInputs && !isScalar(typeName)) <>
        TValid.fail(s"undefined type name $typeName").when(!inOutputs && !isScalar(typeName))
    }

    private def isScalar(typeName: String): Boolean = List("String", "Int", "Boolean").contains(typeName)

    private def needsResolving(field: Config.Field): Boolean =
      field.unsafeSteps.exists(_.nonEmpty) || field.http.isDefined

    private def toArgs(field: Field): TValid[String, List[Blueprint.InputFieldDefinition]] = {
      TValid.foreach(field.args.getOrElse(Map.empty).toList) { case (name, arg) =>
        val ofType = toType(arg)
        for {
          _ <- assertTypeName(ofType.defaultName, isInput = true).trace(name)
        } yield Blueprint.InputFieldDefinition(
          name = arg.modify.flatMap(_.name).getOrElse(name),
          ofType = ofType,
          defaultValue = arg.defaultValue.flatMap(Transcoder.toDynamicValue(_).toOption),
          description = arg.doc,
        )
      }
    }

    private def toDefinitions: TValid[String, List[Blueprint.Definition]] = {
      TValid.foreach(config.graphQL.types.toList) { case (typeName, typeInfo) =>
        val dblUsage = inputTypes.contains(typeName) && outputTypes.contains(typeName)
        for {
          _      <- TValid.fail(s"$typeName cannot be both used both as input and output type").when(dblUsage)
          fields <- toFieldList(typeName, typeInfo)
        } yield {
          val definition = Blueprint.ObjectTypeDefinition(
            name = typeName,
            fields = fields,
            description = typeInfo.doc,
            implements = typeInfo.implements.toList.flatten.map(NamedType(_, true)),
          )
          if (inputTypes.contains(typeName)) toInputObjectTypeDefinition(definition)
          else if (typeInfo.isInterface) toInterfaceDefinition(definition)
          else definition
        }
      }
    }

    private def toFieldDefault(fieldName: String, field: Field): TValid[String, Blueprint.FieldDefinition] = {
      for {
        args   <- toArgs(field)
        ofType <- toType(field)
      } yield Blueprint.FieldDefinition(name = fieldName, args = args, ofType = ofType, description = field.doc)
    }

    private def toFieldList(typeName: String, typeInfo: Type): TValid[String, List[Blueprint.FieldDefinition]] = {
      TValid.foreach(typeInfo.fields.toList) { case (fieldName, field) =>
        (for {
          bField <- toFieldDefault(fieldName, field)
          bField <-
            if (inputTypes.contains(typeName)) TValid.succeed(List(bField))
            else for {
              bField      <- updateUnsafeField(field, bField).trace("@" + UnsafeSteps.directive.name)
              bField      <- updateFieldHttp(field, bField).trace("@" + Http.directive.name)
              mayBeBField <- updateModifyField(field, bField).trace("@" + ModifyField.directive.name)
              bField      <- mayBeBField match {
                case Some(bField) => updateInlineField(typeName, typeInfo, fieldName, field, bField).some
                    .trace("@" + InlineType.directive.name)
                case None         => TValid.none
              }
            } yield bField.toList
        } yield bField).trace(fieldName)
      }.map(_.flatten).trace(typeName)
    }

    private def toHttpResolver(field: Field, http: Operation.Http): TValid[String, DynamicValue ~> DynamicValue] = {
      http.baseURL.orElse(config.server.baseURL) match {
        case Some(baseURL) => TValid.succeed {
            val steps    = field.unsafeSteps.getOrElse(Nil)
            val host     = baseURL.getHost
            val port     = if (baseURL.getPort > 0) baseURL.getPort else 80
            val scheme   = if (baseURL.getProtocol.toLowerCase == "https" || port == 443) Scheme.Https else Scheme.Http
            var endpoint = Endpoint.make(host).withPort(port).withPath(http.path).withScheme(scheme)
              .withQuery(http.query.getOrElse(Map.empty)).withMethod(http.method.getOrElse(Method.GET))
              .withInput(http.input).withOutput(http.output)

            http.body.flatMap(MustacheExpression.syntax.parseString(_).toOption) match {
              case Some(value) => endpoint = endpoint.withBody(value)
              case None        => ()
            }

            // TODO: add unit tests for when we can infer input/output
            val inferOutput = steps.indexOf(http) == steps.length - 1 && endpoint.output.isEmpty

            val inferInput = steps.indexOf(http) == 0 && endpoint.input.isEmpty
            if (inferOutput) endpoint = endpoint.withOutput(Option(toTSchema(field)))
            if (inferInput) endpoint = endpoint.withInput(Option(toTSchema(field.args)))

            val resolver = Lambda.unsafe.fromEndpoint(endpoint)
            http.batchKey match {
              case None      => resolver
              case Some(key) =>
                val baseResolver = resolver.toTyped[Chunk[DynamicValue]].getOrElse(Lambda(Chunk.empty[DynamicValue]))
                  .groupBy(_.pathSeq(http.groupBy.getOrElse(List("id")): _*))
                  .get(Lambda.identity[DynamicValue].path("value", key))
                if (field.isList) baseResolver.map(_.toChunk).toDynamic else baseResolver.flatMap(_.head).toDynamic
            }
          }
        case None          => TValid.fail("No base URL defined in the server configuration")
      }
    }

    /**
     * Converts an object type definition into an input
     * object type definition.
     */
    private def toInputObjectTypeDefinition(
      definition: Blueprint.ObjectTypeDefinition
    ): Blueprint.InputObjectTypeDefinition = {

      val fields = definition.fields.map { field =>
        Blueprint.InputFieldDefinition(
          name = field.name, // field already has the new name
          ofType = field.ofType.withName(field.ofType.defaultName),
          defaultValue = None,
          description = field.description,
        )
      }
      Blueprint.InputObjectTypeDefinition(name = definition.name, fields = fields, description = definition.description)
    }

    private def toInterfaceDefinition(definition: Blueprint.ObjectTypeDefinition): Blueprint.InterfaceTypeDefinition = {
      Blueprint.InterfaceTypeDefinition(
        name = definition.name,
        fields = definition.fields,
        description = definition.description,
      )
    }

    // TODO: Add unit test for mutations
    private def toTSchema(args: Option[Map[String, Arg]]): TSchema = {
      args match {
        case Some(argMap) => TSchema.obj(argMap.map { case (name, arg) =>
            (name, toTSchema(arg.typeOf, arg.isRequired, arg.isList))
          })
        case None         => TSchema.empty
      }
    }

    private def toTSchema(fieldType: String, isRequired: Boolean, isList: Boolean): TSchema = {
      var schema = config.findType(fieldType) match {
        case Some(typeInfo) => TSchema.obj(
            typeInfo.fields.filter { case (_, field) => field.unsafeSteps.forall(_.isEmpty) && field.http.isEmpty }
              .map { case (fieldName, field) => (fieldName, toTSchema(field)) }
          )

        case None => fieldType match {
            case "String"  => TSchema.str
            case "Int"     => TSchema.num
            case "Boolean" => TSchema.bool
            case _         => TSchema.str // TODO: default to string?
          }
      }

      schema = if (isRequired) schema else schema.opt
      schema = if (isList) schema.arr else schema

      schema
    }

    private def toTSchema(field: Field): TSchema = { toTSchema(field.typeOf, field.isRequired, field.isList) }

    private def toType(inputType: Arg): Blueprint.Type = {
      val ofType = Blueprint.NamedType(inputType.typeOf, inputType.isRequired)
      val isList = inputType.isList
      if (isList) Blueprint.ListType(ofType, false) else ofType
    }

    private def toType(field: Field, isRequired: Boolean = true): TValid[Nothing, Blueprint.Type] = {
      val ofType = Blueprint.NamedType(field.typeOf, field.isRequired && isRequired)
      val isList = field.isList
      TValid.succeed(if (isList) Blueprint.ListType(ofType, false) else ofType)
    }

    private def toUnsafeStepsResolver(
      field: Field,
      steps: List[Operation],
    ): TValid[String, Option[DynamicValue ~> DynamicValue]] = {
      if (steps.isEmpty) TValid.none
      else TValid.foreach(steps) {
        case http: Operation.Http           => toHttpResolver(field, http)
        case Operation.Transform(jsonT)     => TValid.succeed(Lambda.identity[DynamicValue].transform(jsonT))
        case Operation.LambdaFunction(func) => TValid.succeed(Lambda.fromFunction(func))
      }.map(_.reduce((f1, f2) => f1 >>> f2)).some
    }

    private def updateFieldHttp(
      field: Field,
      bField: Blueprint.FieldDefinition,
    ): TValid[String, Blueprint.FieldDefinition] = {

      field.http match {
        case Some(http) =>
          if (field.isRequired) { TValid.fail("can not be used with non-nullable fields") }
          else if (field.unsafeSteps.exists(_.nonEmpty)) TValid
            .fail(s"can not be used with @${UnsafeSteps.directive.name}")
          else toHttpResolver(field, http).map(appendResolver(bField, _))
        case _          => TValid.succeed(bField)
      }
    }

    private def updateInlineField(
      typeName: String,
      typeInfo: Type,
      fieldName: String,
      field: Field,
      bField: Blueprint.FieldDefinition,
    ): TValid[String, Blueprint.FieldDefinition] = {
      val inlinedPath = field.inline.map(_.path).getOrElse(Nil)
      val hasIndex    = inlinedPath.exists(_.matches("^\\d+$"))

      def invalidPath: TValid[String, Nothing] =
        TValid.fail(typeName, fieldName, InlineType.directive.name, s"unreachable path: ${inlinedPath.mkString(".")}")

      def loop(
        path: List[String],
        field: Field,
        typeInfo: Type,
        isRequired: Boolean,
      ): TValid[String, Blueprint.Type] = {
        path match {
          case Nil               => toType(field, isRequired)
          case fieldName :: tail =>
            val isNumeric = fieldName.matches("^\\d+$")
            if (isNumeric) loop(tail, field.copy(list = Option(false)), typeInfo, false)
            else for {
              field0 <- TValid.fromOption(typeInfo.fields.get(fieldName)) <> invalidPath
              isRequired0 = isRequired && field0.isRequired
              ofType <-
                if (isScalar(field0.typeOf)) loop(tail, field0, typeInfo, isRequired0)
                else for {
                  typeInfo <- TValid.fromOption(config.findType(field0.typeOf)) <> invalidPath
                  ofType   <- loop(tail, field0, typeInfo, isRequired0)
                } yield if (field.isList) Blueprint.ListType(ofType, isRequired) else ofType
            } yield ofType
        }
      }

      field.inline match {
        case Some(InlineType(path)) => loop(fieldName :: inlinedPath, field, typeInfo, field.isRequired).map(ofType => {
            val nPath    = if (needsResolving(field)) path else "value" :: fieldName :: path
            val resolver =
              if (hasIndex) Lambda.identity[DynamicValue].path(nPath: _*)
              else Lambda.identity[DynamicValue].pathSeq(nPath: _*)
            appendResolver(bField, resolver.toDynamic).copy(ofType = ofType)
          })
        case _                      => TValid.succeed(bField)
      }
    }

    private def updateModifyField(
      field: Field,
      bField: Blueprint.FieldDefinition,
    ): TValid[String, Option[Blueprint.FieldDefinition]] = {
      field.modify match {
        case Some(ModifyField(None, Some(true)))    => TValid.none
        case Some(ModifyField(Some(newName), None)) =>
          val resolverPath = if (bField.resolver.isEmpty) List("value", bField.name) else List()
          val resolver     = Lambda.identity[DynamicValue].path(resolverPath: _*).toDynamic
          TValid.succeed(appendResolver(bField, resolver).copy(name = newName)).some
        case Some(ModifyField(Some(_), Some(_)))    => TValid.fail("can not have both name and omit modifier")
        case _                                      => TValid.succeed(bField).some
      }
    }

    private def updateUnsafeField(
      field: Field,
      bField: Blueprint.FieldDefinition,
    ): TValid[String, Blueprint.FieldDefinition] = {
      if (field.unsafeSteps.exists(_.nonEmpty) && field.isRequired) {
        TValid.fail("can not be used with non-nullable fields")
      } else field.unsafeSteps match {
        case Some(steps) => toUnsafeStepsResolver(field, steps).map {
            case None           => bField
            case Some(resolver) => appendResolver(bField, resolver)
          }
        case None        => TValid.succeed(bField)
      }
    }
  }
}
