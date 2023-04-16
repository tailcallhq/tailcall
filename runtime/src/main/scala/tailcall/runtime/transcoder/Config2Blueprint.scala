package tailcall.runtime.transcoder

import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.internal.TValid
import tailcall.runtime.lambda._
import tailcall.runtime.model.Config._
import tailcall.runtime.model._
import zio.schema.DynamicValue

trait Config2Blueprint {

  /**
   * Encodes a config into a Blueprint.
   */
  final def toBlueprint(config: Config): TValid[Nothing, Blueprint] = {
    val rootSchema = Blueprint
      .SchemaDefinition(query = config.graphQL.schema.query, mutation = config.graphQL.schema.mutation)

    val outputTypes    = getOutputTypes(config).toSet
    val inputTypes     = getInputTypes(config).toSet
    val inputTypeNames = inputTypes.map { name =>
      if (outputTypes.contains(name)) name -> (name + "Input") else name -> name
    }.toMap

    val definitions: List[Blueprint.Definition] = config.graphQL.types.toList.flatMap { case (name, typeInfo) =>
      val fields = toFieldList(config, inputTypeNames, typeInfo)

      // NOTE: Should create a list of definitions
      // There should be an object type or a list of input object type
      val definition      = Blueprint.ObjectTypeDefinition(name = name, fields = fields, description = typeInfo.doc)
      val inputDefinition = toInputObjectTypeDefinition(definition, inputTypeNames)
      if (outputTypes.contains(name) && inputTypes.contains(name)) List(definition, inputDefinition)
      else if (inputTypes.contains(name)) inputDefinition :: Nil
      else definition :: Nil
    }

    TValid.succeed(Blueprint(rootSchema :: definitions))
  }

  private def toFieldList(
    config: Config,
    inputTypeNames: Map[String, String],
    typeInfo: Type,
  ): List[Blueprint.FieldDefinition] = {

    typeInfo.fields.toList.filter(!_._2.modify.flatMap(_.omit).getOrElse(false)).map { case (name, field) =>
      val args     = toArgs(field, inputTypeNames)
      val ofType   = toType(field)
      val resolver = toResolver(config, name, field)

      Blueprint.FieldDefinition(
        name = field.modify.flatMap(_.rename).getOrElse(name),
        args = args,
        ofType = ofType,
        resolver = resolver.map(Lambda.fromFunction(_)),
        description = field.doc,
      )
    }

  }

  private def toArgs(field: Field, inputTypeNames: Map[String, String]): List[Blueprint.InputFieldDefinition] = {
    field.args.getOrElse(Map.empty).toList.map { case (name, arg) =>
      val ofType = toType(arg)

      val prefixedOfType: Blueprint.Type = inputTypeNames.get(ofType.defaultName) match {
        case Some(name) => ofType.withName(name)
        case None       => ofType
      }
      Blueprint.InputFieldDefinition(
        name = arg.modify.flatMap(_.rename).getOrElse(name),
        ofType = prefixedOfType,
        defaultValue = arg.defaultValue.flatMap(Transcoder.toDynamicValue(_).toOption),
        description = arg.doc,
      )
    }
  }

  /**
   * Types are input types if they are used as arguments to
   * a field OR if the are the return types of a field
   * defined in an input type.
   */
  final private def getInputTypes(config: Config): List[String] = {

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
   * Goes over every possible object type and creates a map
   * of type name to whether it's an input type or not.
   */
  final private def getOutputTypes(config: Config): List[String] = {
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

  private def toInputObjectTypeDefinition(
    definition: Blueprint.ObjectTypeDefinition,
    inputNames: Map[String, String],
  ): Blueprint.InputObjectTypeDefinition = {
    val fields = definition.fields.map { field =>
      Blueprint.InputFieldDefinition(
        name = field.name,
        ofType = field.ofType.withName(inputNames.getOrElse(field.ofType.defaultName, field.ofType.defaultName)),
        defaultValue = None,
        description = field.description,
      )
    }
    Blueprint.InputObjectTypeDefinition(
      name = inputNames.getOrElse(definition.name, definition.name),
      fields = fields,
      description = definition.description,
    )
  }

  final private def toResolver(
    config: Config,
    field: Field,
    http: Step.Http,
  ): TValid[String, DynamicValue ~>> DynamicValue] = {
    config.server.baseURL match {
      case Some(baseURL) => TValid.succeed { input =>
          val steps = field.steps.getOrElse(Nil)
          val host  = baseURL.getHost
          val port  = if (baseURL.getPort > 0) baseURL.getPort else 80

          var endpoint = Endpoint.make(host).withPort(port).withPath(http.path)
            .withProtocol(if (port == 443) Scheme.Https else Scheme.Http).withMethod(http.method.getOrElse(Method.GET))
            .withInput(http.input).withOutput(http.output)
          http.body.flatMap(Mustache.syntax.parseString(_).toOption) match {
            case Some(value) => endpoint = endpoint.withBody(value)
            case None        => ()
          }

          // TODO: add unit tests for when we can infer input/output
          val inferOutput = steps.indexOf(http) == steps.length - 1 && endpoint.output.isEmpty
          val inferInput  = steps.indexOf(http) == 0 && endpoint.input.isEmpty
          if (inferOutput) endpoint = endpoint.withOutput(Option(toTSchema(config, field)))
          if (inferInput) endpoint = endpoint.withInput(Option(toTSchema(config, field.args)))

          input >>> Lambda.unsafe.fromEndpoint(endpoint)
        }
      case None          => TValid.fail("No base URL defined in the server configuration")
    }
  }

  // TODO: change it to Lambda[DynamicValue, DyanmicValue]
  type Resolver = DynamicValue ~>> DynamicValue

  final private def toResolver(config: Config, field: Field, step: Step): TValid[String, Resolver] = {
    step match {
      case http @ Step.Http(_, _, _, _, _) => toResolver(config, field, http)
      case Step.Transform(jsonT)           => TValid.succeed(dynamic => dynamic.transform(jsonT))
      case Step.LambdaFunction(func)       => TValid.succeed(func)
    }
  }

  final private def toResolver(config: Config, name: String, field: Field): Option[Resolver] = {
    field.steps match {
      case None        => field.modify.flatMap(_.rename) match {
          case Some(_) => Option(input => input.path("value", name).toDynamic)
          case None    => None
        }
      case Some(steps) => TValid.foreach(steps.map(toResolver(config, field, _)))(identity(_))
          .map(_.reduce((f1, f2) => a => f2(f1(a)))).toOption
    }
  }

  // TODO: Add unit test for mutations
  final private def toTSchema(config: Config, args: Option[Map[String, Arg]]): TSchema = {
    args match {
      case Some(argMap) => TSchema.obj(argMap.map { case (name, arg) =>
          (name, toTSchema(config, arg.typeOf, arg.isRequired, arg.isList))
        })
      case None         => TSchema.empty
    }
  }

  final private def toTSchema(config: Config, fieldName: String, isRequired: Boolean, isList: Boolean): TSchema = {
    var schema = config.graphQL.types.get(fieldName) match {
      case Some(typeInfo) => TSchema.obj(typeInfo.fields.filter(_._2.steps.isEmpty).map { case (fieldName, field) =>
          (fieldName, toTSchema(config, field))
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

  final private def toTSchema(config: Config, field: Field): TSchema = {
    toTSchema(config, field.typeOf, field.isRequired, field.isList)
  }

  final private def toType(inputType: Arg): Blueprint.Type = {
    val ofType = Blueprint.NamedType(inputType.typeOf, inputType.isRequired)
    val isList = inputType.isList
    if (isList) Blueprint.ListType(ofType, false) else ofType
  }

  final private def toType(field: Field): Blueprint.Type = {
    val ofType = Blueprint.NamedType(field.typeOf, field.isRequired)
    val isList = field.isList
    if (isList) Blueprint.ListType(ofType, false) else ofType
  }
}
