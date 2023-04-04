package tailcall.runtime.transcoder

import caliban.InputValue
import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.internal.{JsonSchema, TValid}
import tailcall.runtime.model.Config._
import tailcall.runtime.model._
import tailcall.runtime.remote.Remote
import zio.json.ast.Json
import zio.json.{DecoderOps, EncoderOps}
import zio.schema.{DynamicValue, Schema}

trait Config2Blueprint {

  implicit final private def jsonSchema: Schema[Json] = JsonSchema.schema

  /**
   * Encodes a config into a Blueprint.
   * @param config
   *   the config to encode
   * @param encodeDirectives
   *   if true, annotations and steps will be encoded as
   *   directives
   * @return
   */
  final def toBlueprint(config: Config, encodeDirectives: Boolean = false): TValid[Nothing, Blueprint] = {
    val rootSchema = Blueprint.SchemaDefinition(
      query = config.graphQL.schema.query,
      mutation = config.graphQL.schema.mutation,
      directives = if (encodeDirectives) toServerDirective(config).toList else Nil,
    )

    val outputTypes    = getOutputTypes(config).toSet
    val inputTypes     = getInputTypes(config).toSet
    val inputTypeNames = inputTypes.map { name =>
      if (outputTypes.contains(name)) name -> (name + "Input") else name -> name
    }.toMap

    val definitions: List[Blueprint.Definition] = config.graphQL.types.toList.flatMap { case (name, typeInfo) =>
      val bFields: List[Blueprint.FieldDefinition] = {
        typeInfo.fields.toList.map { case (name, field) =>
          val args: List[Blueprint.InputFieldDefinition] = {
            field.args.getOrElse(Map.empty).toList.map { case (name, arg) =>
              val ofType = toType(arg)

              val prefixedOfType: Blueprint.Type = inputTypeNames.get(ofType.defaultName) match {
                case Some(name) => ofType.withName(name)
                case None       => ofType
              }
              Blueprint
                .InputFieldDefinition(name = name, ofType = prefixedOfType, defaultValue = None, description = arg.doc)
            }
          }

          val ofType = toType(field)

          val resolver = toResolver(config, field.steps.getOrElse(Nil), field)

          var directives = List.empty[Blueprint.Directive]
          if (encodeDirectives) {
            directives = toDirective(field.steps.getOrElse(Nil)).toList
            directives = field.rename match {
              case Some(value) => FieldAnnotation.rename(value).toDirective :: directives
              case None        => directives
            }
          }

          Blueprint.FieldDefinition(
            name = name,
            args = args,
            ofType = ofType,
            resolver = resolver.map(Remote.toLambda(_)),
            directives = directives,
            description = field.doc,
          )
        }
      }

      // NOTE: Should create a list of definitions
      // There should be an object type or a list of input object type
      val definition      = Blueprint.ObjectTypeDefinition(name = name, fields = bFields, description = typeInfo.doc)
      val inputDefinition = toInputObjectTypeDefinition(definition, inputTypeNames)
      if (outputTypes.contains(name) && inputTypes.contains(name)) List(definition, inputDefinition)
      else if (inputTypes.contains(name)) inputDefinition :: Nil
      else definition :: Nil
    }

    TValid.succeed(Blueprint(rootSchema :: definitions))
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

  final private def toServerDirective(config: Config): Option[Blueprint.Directive] = {
    if (config.server.isEmpty) None
    else {
      val map        = config.server.toJson.fromJson[Map[String, InputValue]]
      val serverArgs = (map match {
        case Left(_)     => TValid.succeed(Nil)
        case Right(args) => TValid.foreach(args.toList) { case (k, v) => Transcoder.toDynamicValue(v).map(k -> _) }
      }).map(_.toMap).toOption

      serverArgs.map(args => Blueprint.Directive(name = "server", arguments = args))
    }
  }

  final private def toDirective(step: List[Step]): Option[Blueprint.Directive] = {
    // TODO: should fail on error
    val (errors, jsons) = step.map(_.toJsonAST).partitionMap(identity(_))
    if (errors.nonEmpty || jsons.isEmpty) None
    else Transcoder.toDynamicValue(Json.Arr(jsons: _*)).toEither match {
      case Left(_)             => None
      case Right(dynamicValue) => Option(Blueprint.Directive(name = "steps", arguments = Map("value" -> dynamicValue)))
    }
  }

  final private def toEndpoint(http: Step.Http, host: String, port: Int): Endpoint = {
    Endpoint.make(host).withPort(port).withPath(http.path).withProtocol(if (port == 443) Scheme.Https else Scheme.Http)
      .withMethod(http.method.getOrElse(Method.GET)).withInput(http.input).withOutput(http.output)
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

  final private def toRemoteMap(lookup: Remote[DynamicValue], map: Map[String, List[String]]): Remote[DynamicValue] =
    map.foldLeft(Remote(Map.empty[String, DynamicValue])) { case (to, (key, path)) =>
      lookup.path(path: _*).map(value => to.put(Remote(key), value)).getOrElse(to)
    }.toDynamic

  final private def toResolver(
    config: Config,
    steps: List[Step],
    field: Field,
  ): Option[Remote[DynamicValue] => Remote[DynamicValue]] =
    steps match {
      case Nil => None

      case steps => config.server.baseURL match {
          // TODO: should fail if Http is used without server.host
          case None if steps.exists(_.isInstanceOf[Step.Http]) => None
          case option                                          => option.map { baseURL =>
              steps.map[Remote[DynamicValue] => Remote[DynamicValue]] {
                case http @ Step.Http(_, _, _, _) => input =>
                    val host               = baseURL.getHost
                    val port               = if (baseURL.getPort > 0) baseURL.getPort else 80
                    val endpoint           = toEndpoint(http, host, port)
                    val inferOutput        = steps.indexOf(http) == steps.length - 1 && endpoint.output.isEmpty
                    val endpointWithOutput =
                      if (inferOutput) endpoint.withOutput(Option(toTSchema(config, field))) else endpoint
                    Remote.fromEndpoint(endpointWithOutput, input)
                case Step.Constant(json)          => _ => Remote(json).toDynamic
                case Step.ObjPath(map)            => input => toRemoteMap(input, map)
              }.reduce((a, b) => r => b(a(r)))
            }
        }
    }

  final private def toTSchema(config: Config, field: Field): TSchema = {
    var schema = config.graphQL.types.get(field.typeOf) match {
      case Some(typeInfo) => TSchema.obj(typeInfo.fields.filter(_._2.steps.isEmpty).map { case (fieldName, field) =>
          (fieldName, toTSchema(config, field))
        })

      case None => field.typeOf match {
          case "String"  => TSchema.string
          case "Int"     => TSchema.int
          case "Boolean" => TSchema.bool
          case _         => TSchema.string // TODO: default to string?
        }
    }

    schema = if (field.isRequired) schema else schema.opt
    schema = if (field.isList) schema.arr else schema

    schema
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
