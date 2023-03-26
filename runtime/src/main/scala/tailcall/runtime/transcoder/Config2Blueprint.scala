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

  final private def toType(field: Field): Blueprint.Type = {
    val ofType = Blueprint.NamedType(field.typeOf, field.isRequired.getOrElse(false))
    val isList = field.isList.getOrElse(false)
    if (isList) Blueprint.ListType(ofType, false) else ofType
  }

  final private def toType(inputType: Argument): Blueprint.Type = {
    val ofType = Blueprint.NamedType(inputType.typeOf, inputType.isRequired.getOrElse(false))
    val isList = inputType.isList.getOrElse(false)
    if (isList) Blueprint.ListType(ofType, false) else ofType
  }

  final private def toTSchema(config: Config, field: Field): TSchema = {
    config.graphQL.types.get(field.typeOf) match {
      case Some(value) =>
        val schema = TSchema.obj(value.toList.filter(_._2.steps.isEmpty).map { case (fieldName, field) =>
          TSchema.Field(fieldName, toTSchema(config, field))
        })

        if (field.isList.getOrElse(false)) schema.arr else schema

      case None => field.typeOf match {
          case "String"  => TSchema.string
          case "Int"     => TSchema.int
          case "Boolean" => TSchema.bool
          case _         => TSchema.NULL
        }
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

  final def toDirective(step: List[Step]): Option[Blueprint.Directive] = {
    // TODO: should fail on error
    val (errors, jsons) = step.map(_.toJsonAST).partitionMap(identity(_))
    if (errors.nonEmpty || jsons.isEmpty) None
    else Transcoder.toDynamicValue(Json.Arr(jsons: _*)).toEither match {
      case Left(_)             => None
      case Right(dynamicValue) => Option(Blueprint.Directive(name = "steps", arguments = Map("value" -> dynamicValue)))
    }
  }

  final private def toDirective(config: Config): Option[Blueprint.Directive] = {
    val map = config.server.toJson.fromJson[Map[String, InputValue]]

    val serverArgs = (map match {
      case Left(_)     => TValid.succeed(Nil)
      case Right(args) => TValid.foreach(args.toList) { case (k, v) => Transcoder.toDynamicValue(v).map(k -> _) }
    }).map(_.toMap).toOption

    serverArgs.map(args => Blueprint.Directive(name = "server", arguments = args))

  }

  final def toBlueprint(config: Config): TValid[Nothing, Blueprint] = {
    val rootSchema = Blueprint.SchemaDefinition(
      query = config.graphQL.schema.query,
      mutation = config.graphQL.schema.mutation,
      directives = toDirective(config).toList,
    )

    val definitions: List[Blueprint.Definition] = config.graphQL.types.toList.map { case (name, fields) =>
      val bFields: List[Blueprint.FieldDefinition] = {
        fields.toList.map { case (name, field) =>
          val args: List[Blueprint.InputValueDefinition] = {
            field.args.getOrElse(Map.empty).toList.map { case (name, inputType) =>
              Blueprint.InputValueDefinition(name, toType(inputType), None)
            }
          }

          val ofType = toType(field)

          val resolver = toResolver(config, field.steps.getOrElse(Nil), field)

          Blueprint.FieldDefinition(
            name = name,
            args = args,
            ofType = ofType,
            resolver = resolver.map(Remote.toLambda(_)),
            directives = Nil,
          )
        }
      }

      Blueprint.ObjectTypeDefinition(name = name, fields = bFields)
    }

    TValid.succeed(Blueprint(rootSchema :: definitions))
  }
}
