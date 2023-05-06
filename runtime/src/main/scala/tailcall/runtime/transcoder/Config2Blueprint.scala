package tailcall.runtime.transcoder

import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.internal.TValid
import tailcall.runtime.lambda.Syntax._
import tailcall.runtime.lambda._
import tailcall.runtime.model.Blueprint.FieldDefinition
import tailcall.runtime.model.Config._
import tailcall.runtime.model.Mustache.MustacheExpression
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model._
import zio.Chunk
import zio.schema.DynamicValue

trait Config2Blueprint {
  def toBlueprint(config: Config): TValid[String, Blueprint] = Config2Blueprint.Live(config).toBlueprint
}

object Config2Blueprint {
  final case class Live(config: Config) {
    private val outputTypes    = getOutputTypes.toSet
    private val inputTypes     = getInputTypes.toSet
    private val inputTypeNames = inputTypes.map { name =>
      if (outputTypes.contains(name)) name -> (name + "Input") else name -> name
    }.toMap

    /**
     * Encodes a config into a Blueprint.
     */
    def toBlueprint: TValid[String, Blueprint] = {
      val rootSchema = Blueprint
        .SchemaDefinition(query = config.graphQL.schema.query, mutation = config.graphQL.schema.mutation)

      for {
        definitions <- toDefinitions
      } yield Blueprint(rootSchema :: definitions, Blueprint.Server(config.server.timeout))

    }

    private def appendBatchResolver(
      bField: Blueprint.FieldDefinition,
      f: DynamicValue ~> DynamicValue,
    ): FieldDefinition =
      bField.batchResolver match {
        case Some(g) => bField.copy(resolver = Option(g >>> f))
        case None    => bField.copy(resolver = Option(f))
      }

    private def appendResolver(
      bField: Blueprint.FieldDefinition,
      f: DynamicValue ~> DynamicValue,
    ): Blueprint.FieldDefinition =
      bField.resolver match {
        case Some(g) => bField.copy(resolver = Option(g >>> f))
        case None    => bField.copy(resolver = Option(f))
      }

    /**
     * Types are input types if they are used as arguments
     * to a field OR if the are the return types of a field
     * defined in an input type.
     */
    private def getInputTypes: List[String] = {

      def collectReturnTypes(name: String, returnTypes: List[String]): List[String] = {
        if (returnTypes.contains(name)) returnTypes
        else config.graphQL.types.get(name) match {
          case Some(typeInfo) => typeInfo.returnTypes.flatMap(collectReturnTypes(_, name :: returnTypes))
          case None           => returnTypes
        }
      }

      config.graphQL.types.values.toList.flatMap(_.fields.values.toList)
        .flatMap(_.args.getOrElse(Map.empty).values.toList).map(_.typeOf).flatMap(collectReturnTypes(_, Nil))
    }

    /**
     * Goes over every possible object type and creates a
     * map of type name to whether it's an input type or
     * not.
     */
    private def getOutputTypes: List[String] = {
      def loop(name: String, result: List[String]): List[String] = {
        if (result.contains(name)) result
        else config.graphQL.types.get(name) match {
          case Some(typeInfo) => typeInfo.fields.values.toList
              .flatMap[String](field => loop(field.typeOf, name :: result))
          case None           => result
        }
      }

      val types = config.graphQL.schema.query.toList ++ config.graphQL.schema.mutation.toList
      types ++ types.foldLeft(List.empty[String]) { case (list, name) => loop(name, list) }
    }

    private def toArgs(field: Field): List[Blueprint.InputFieldDefinition] = {
      field.args.getOrElse(Map.empty).toList.map { case (name, arg) =>
        val ofType = toType(arg)

        val prefixedOfType: Blueprint.Type = inputTypeNames.get(ofType.defaultName) match {
          case Some(name) => ofType.withName(name)
          case None       => ofType
        }
        Blueprint.InputFieldDefinition(
          name = arg.modify.flatMap(_.name).getOrElse(name),
          ofType = prefixedOfType,
          defaultValue = arg.defaultValue.flatMap(Transcoder.toDynamicValue(_).toOption),
          description = arg.doc,
        )
      }
    }

    private def toDefinitions: TValid[String, List[Blueprint.Definition]] = {
      TValid.foreach(config.graphQL.types.toList) { case (name, typeInfo) =>
        for {
          fields <- toFieldList(name, typeInfo)
        } yield {

          // NOTE: Should create a list of definitions
          // There should be an object type or a list of input object type
          val definition      = Blueprint.ObjectTypeDefinition(name = name, fields = fields, description = typeInfo.doc)
          val inputDefinition = toInputObjectTypeDefinition(definition)

          if (outputTypes.contains(name) && inputTypes.contains(name)) List(definition, inputDefinition)
          else if (inputTypes.contains(name)) inputDefinition :: Nil
          else definition :: Nil
        }
      }.map(_.flatten)
    }

    private def toFieldDefault(fieldName: String, field: Field): TValid[String, Blueprint.FieldDefinition] = {
      val args = toArgs(field)
      toType(field).map(ofType =>
        Blueprint.FieldDefinition(name = fieldName, args = args, ofType = ofType, description = field.doc)
      )
    }

    private def toFieldList(typeName: String, typeInfo: Type): TValid[String, List[Blueprint.FieldDefinition]] =
      TValid.foreach(typeInfo.fields.toList) { case (fieldName, field) =>
        for {
          bField      <- toFieldDefault(fieldName, field)
          bField      <- updateUnsafeField(typeName, field, bField)
          bField      <- updateFieldHttp(typeName, field, bField)
          mayBeBField <- updateModifyField(field, bField, inputTypeNames.contains(typeName))
          bField      <- mayBeBField match {
            case Some(bField) => updateInlineField(typeInfo, fieldName, field, bField).some
            case None         => TValid.none
          }
        } yield bField.toList
      }.map(_.flatten)

    private def toHttpResolver(field: Field, http: Operation.Http): TValid[String, DynamicValue ~> DynamicValue] = {
      config.server.baseURL match {
        case Some(baseURL) => TValid.succeed {
            val steps    = field.unsafeSteps.getOrElse(Nil)
            val host     = baseURL.getHost
            val port     = if (baseURL.getPort > 0) baseURL.getPort else 80
            val scheme   = if (baseURL.getProtocol.toLowerCase == "https" || port == 443) Scheme.Https else Scheme.Http
            var endpoint = Endpoint.make(host).withPort(port).withPath(http.path).withProtocol(scheme)
              .withQuery(http.query.getOrElse(Map.empty)).withMethod(http.method.getOrElse(Method.GET))
              .withInput(http.input).withOutput(http.output)

            http.body.flatMap(MustacheExpression.syntax.parseString(_).toOption) match {
              case Some(value) => endpoint = endpoint.withBody(value)
              case None        => ()
            }

            // TODO: add unit tests for when we can infer input/output
            val inferOutput = steps.indexOf(http) == steps.length - 1 && endpoint.output.isEmpty
            val inferInput  = steps.indexOf(http) == 0 && endpoint.input.isEmpty
            if (inferOutput) endpoint = endpoint.withOutput(Option(toTSchema(field)))
            if (inferInput) endpoint = endpoint.withInput(Option(toTSchema(field.args)))

            val resolver = Lambda.unsafe.fromEndpoint(endpoint)
            http.batchKey match {
              case None      => resolver
              case Some(key) =>
                // FIXME: handle single values
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
          ofType = field.ofType.withName(inputTypeNames.getOrElse(field.ofType.defaultName, field.ofType.defaultName)),
          defaultValue = None,
          description = field.description,
        )
      }
      Blueprint.InputObjectTypeDefinition(
        name = inputTypeNames.getOrElse(definition.name, definition.name),
        fields = fields,
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

      var schema = config.graphQL.types.get(fieldType) match {
        case Some(typeInfo) => TSchema.obj(
            typeInfo.fields.filter { case (_, field) =>
              field.unsafeSteps.exists(_.isEmpty) && field.http.exists(_.exists(_.input.isEmpty))
            }.map { case (fieldName, field) => (fieldName, toTSchema(field)) }
          )

        case None => fieldType match {
            case "String"  => TSchema.string
            case "Int"     => TSchema.num
            case "Boolean" => TSchema.bool
            case _         => TSchema.string // TODO: default to string?
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

    private def toType(field: Field): TValid[Nothing, Blueprint.Type] = {
      val ofType = Blueprint.NamedType(field.typeOf, field.isRequired)
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
      typeName: String,
      field: Field,
      bField: Blueprint.FieldDefinition,
    ): TValid[String, Blueprint.FieldDefinition] = {

      field.http match {
        case Some(httpList) if httpList.nonEmpty =>
          if (field.isRequired) TValid
            .fail(s"`${typeName}.${bField.name}` has an http operation hence can not be non-nullable")
          else if (field.unsafeSteps.exists(_.nonEmpty)) TValid
            .fail(s"${typeName}.${bField.name} can not have unsafe and http operations together")
          else TValid.fold(httpList, bField) { case (bField, http) =>
            toHttpResolver(field, http).map { resolver =>
              if (http.batchKey.nonEmpty) appendBatchResolver(bField, resolver) else appendResolver(bField, resolver)
            }
          }
        case _                                   => TValid.succeed(bField)
      }
    }

    private def updateInlineField(
      typeInfo: Type,
      fieldName: String,
      field: Field,
      bField: Blueprint.FieldDefinition,
    ): TValid[String, Blueprint.FieldDefinition] = {
      val inlinedPath = field.inline.map(_.path).getOrElse(Nil)
      val hasIndex    = inlinedPath.exists(_.matches("^\\d+$"))
      def loop(path: List[String], field: Field, typeInfo: Type): TValid[String, Blueprint.Type] = {
        path match {
          case Nil               => toType(field)
          case fieldName :: tail =>
            val fieldNameIsIndex = fieldName.matches("^\\d+$")
            if (fieldNameIsIndex) loop(tail, field, typeInfo).map {
              case Blueprint.ListType(ofType, _) => ofType
              case ofType                        => ofType
            }
            else {
              def invalidKey  =
                s"Inlining path [${inlinedPath
                    .mkString(", ")}] failed because field '${fieldName}' was not found on type '${field.typeOf}'"
              def invalidPath = s"Inlining path ${inlinedPath.mkString(", ")} is invalid for field ${fieldName}"

              for {
                field    <- TValid.fromOption(typeInfo.fields.get(fieldName), invalidKey)
                typeInfo <- TValid.fromOption(config.graphQL.types.get(field.typeOf), invalidPath)
                ofType   <- loop(tail, field, typeInfo)
              } yield if (field.isList && tail.nonEmpty) Blueprint.ListType(ofType, field.isRequired) else ofType
            }
        }
      }

      field.inline match {
        case Some(InlineType(path)) => loop(fieldName :: inlinedPath, field, typeInfo).map(ofType => {
            val resolver =
              if (hasIndex) Lambda.identity[DynamicValue].path(path: _*)
              else Lambda.identity[DynamicValue].pathSeq(path: _*)
            appendResolver(bField, resolver.toDynamic).copy(ofType = ofType)
          })
        case _                      => TValid.succeed(bField)
      }
    }

    private def updateModifyField(
      field: Field,
      bField: Blueprint.FieldDefinition,
      isInputType: Boolean,
    ): TValid[String, Option[Blueprint.FieldDefinition]] = {
      field.modify match {
        case Some(ModifyField(None, Some(true)))    => TValid.none
        case Some(ModifyField(Some(newName), None)) =>
          if (isInputType) TValid.succeed(bField).some
          else {
            val resolverPath = if (bField.resolver.isEmpty) List("value", bField.name) else List()
            val resolver     = Lambda.identity[DynamicValue].path(resolverPath: _*).toDynamic
            TValid.succeed(appendResolver(bField, resolver).copy(name = newName)).some
          }
        case Some(ModifyField(Some(_), Some(_)))    => TValid
            .fail(s"Cannot have both name and omit modifier on field ${bField.name}")
        case _                                      => TValid.succeed(bField).some
      }
    }

    private def updateUnsafeField(
      typeName: String,
      field: Field,
      bField: Blueprint.FieldDefinition,
    ): TValid[String, Blueprint.FieldDefinition] = {
      if (field.unsafeSteps.exists(_.nonEmpty) && field.isRequired) TValid
        .fail(s"`${typeName}.${bField.name}` has an unsafe operation hence can not be non-nullable")
      else field.unsafeSteps match {
        case Some(steps) => toUnsafeStepsResolver(field, steps).map {
            case None           => bField
            case Some(resolver) => appendResolver(bField, resolver)
          }
        case None        => TValid.succeed(bField)
      }
    }
  }
}
