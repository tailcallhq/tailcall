package tailcall.runtime.transcoder

import caliban.InputValue
import tailcall.runtime.ast.{Blueprint, Endpoint, TSchema}
import tailcall.runtime.dsl.Config
import tailcall.runtime.dsl.Config._
import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.internal.TValid
import tailcall.runtime.remote.Remote
import zio.json.ast.Json
import zio.json.{DecoderOps, EncoderOps}
import zio.schema.{DynamicValue, Schema}

trait Config2Blueprint {

  implicit final private def jsonSchema: Schema[Json] =
    Schema[DynamicValue].transformOrFail[Json](Transcoder.toJson(_).toEither, Transcoder.toDynamicValue(_).toEither)

  final def toBlueprint(config: Config, encodeSteps: Boolean = false): TValid[Nothing, Blueprint] = {
    val rootSchema = Blueprint.SchemaDefinition(
      query = config.graphQL.schema.query,
      mutation = config.graphQL.schema.mutation,
      directives = toDirective(config).toList,
    )

    val outputTypes = getOutputTypes(config).toSet

    val definitions: List[Blueprint.Definition] = config.graphQL.types.toList.map { case (name, typeInfo) =>
      val bFields: List[Blueprint.FieldDefinition] = {
        typeInfo.fields.toList.map { case (name, field) =>
          val args: List[Blueprint.InputFieldDefinition] = {
            field.args.getOrElse(Map.empty).toList.map { case (name, arg) =>
              Blueprint
                .InputFieldDefinition(name = name, ofType = toType(arg), defaultValue = None, description = arg.doc)
            }
          }

          val ofType = toType(field)

          val resolver = toResolver(config, field.steps.getOrElse(Nil), field)

          Blueprint.FieldDefinition(
            name = name,
            args = args,
            ofType = ofType,
            resolver = resolver.map(Remote.toLambda(_)),
            directives = if (encodeSteps) toDirective(field.steps.getOrElse(Nil)).toList else Nil,
            description = field.doc,
          )
        }
      }

      // NOTE: Should create a list of definitions
      // There should be an object type or a list of input object type
      val definition = Blueprint.ObjectTypeDefinition(name = name, fields = bFields, description = typeInfo.doc)
      if (outputTypes.contains(name)) { definition }
      else definition.toInput
    }

    TValid.succeed(Blueprint(rootSchema :: definitions))
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

  final private def toDirective(config: Config): Option[Blueprint.Directive] = {
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
      case Some(typeInfo) => TSchema.obj(typeInfo.fields.toList.filter(_._2.steps.isEmpty).map {
          case (fieldName, field) => TSchema.Field(fieldName, toTSchema(config, field))
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
