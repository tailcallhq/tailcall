package tailcall.runtime.transcoder

import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.internal.TValid
import tailcall.runtime.lambda._
import tailcall.runtime.model.Config._
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model._
import zio.schema.DynamicValue

trait Config2Blueprint {

  def toBlueprint(config: Config): TValid[String, Blueprint] = Config2Blueprint.Live(config).toBlueprint
}

object Config2Blueprint {
  // TODO: change it to Lambda[DynamicValue, DyanmicValue]
  type Resolver = DynamicValue ~>> DynamicValue

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

    private def toFieldList(typeName: String, typeInfo: Type): TValid[String, List[Blueprint.FieldDefinition]] =
      TValid.foreach(typeInfo.fields.toList.filter(!_._2.modify.flatMap(_.omit).getOrElse(false))) {
        case (fieldName, field) =>
          val args   = toArgs(field)
          val ofType = toType(field)

          for {
            resolver <- toResolver(typeName, fieldName, field, inputTypeNames.contains(typeName))
          } yield Blueprint.FieldDefinition(
            name = field.modify.flatMap(_.name).getOrElse(fieldName),
            args = args,
            ofType = ofType,
            resolver = resolver.map(Lambda.fromFunction(_)),
            description = field.doc,
          )
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

    private def toResolver(field: Field, http: Operation.Http): TValid[String, DynamicValue ~>> DynamicValue] = {
      config.server.baseURL match {
        case Some(baseURL) => TValid.succeed { input =>
            val steps = field.unsafeSteps.getOrElse(Nil)
            val host  = baseURL.getHost
            val port  = if (baseURL.getPort > 0) baseURL.getPort else 80

            var endpoint = Endpoint.make(host).withPort(port).withPath(http.path)
              .withProtocol(if (port == 443) Scheme.Https else Scheme.Http)
              .withMethod(http.method.getOrElse(Method.GET)).withInput(http.input).withOutput(http.output)

            http.body.flatMap(Mustache.syntax.parseString(_).toOption) match {
              case Some(value) => endpoint = endpoint.withBody(value)
              case None        => ()
            }

            // TODO: add unit tests for when we can infer input/output
            val inferOutput = steps.indexOf(http) == steps.length - 1 && endpoint.output.isEmpty
            val inferInput  = steps.indexOf(http) == 0 && endpoint.input.isEmpty
            if (inferOutput) endpoint = endpoint.withOutput(Option(toTSchema(field)))
            if (inferInput) endpoint = endpoint.withInput(Option(toTSchema(field.args)))

            input >>> Lambda.unsafe.fromEndpoint(endpoint)
          }
        case None          => TValid.fail("No base URL defined in the server configuration")
      }
    }

    private def toResolver(field: Field, step: Operation): TValid[String, Resolver] = {
      step match {
        case http @ Operation.Http(_, _, _, _, _) => toResolver(field, http)
        case Operation.Transform(jsonT)           => TValid.succeed(dynamic => dynamic.transform(jsonT))
        case Operation.LambdaFunction(func)       => TValid.succeed(func)
      }
    }

    private def toResolver(
      typeName: String,
      fieldName: String,
      field: Field,
      isInputType: Boolean,
    ): TValid[String, Option[Resolver]] = {
      if (field.http.nonEmpty && field.unsafeSteps.forall(_.nonEmpty)) TValid
        .fail(s"type ${typeName} with field ${fieldName} can not have both an unsafe and an http operations together")
      else TValid.succeed {
        field.unsafeSteps match {
          case None => field.modify.flatMap(_.name) match {
              case Some(newName) =>
                val finalName = if (isInputType) newName else fieldName
                Option(input => input.path("value", finalName).toDynamic)
              case None          => None
            }

          case Some(steps) => TValid.foreach(steps.map(toResolver(field, _)))(identity(_))
              .map(_.reduce((f1, f2) => a => f2(f1(a)))).toOption
        }
      }
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

    private def toTSchema(fieldName: String, isRequired: Boolean, isList: Boolean): TSchema = {
      var schema = config.graphQL.types.get(fieldName) match {
        case Some(typeInfo) => TSchema.obj(typeInfo.fields.filter(_._2.unsafeSteps.isEmpty).map {
            case (fieldName, field) => (fieldName, toTSchema(field))
          })

        case None => fieldName match {
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

    private def toType(field: Field): Blueprint.Type = {
      val ofType = Blueprint.NamedType(field.typeOf, field.isRequired)
      val isList = field.isList
      if (isList) Blueprint.ListType(ofType, false) else ofType
    }
  }
}
