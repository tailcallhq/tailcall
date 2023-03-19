package tailcall.runtime.transcoder

import tailcall.runtime.ast.{Blueprint, Endpoint, TSchema}
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.json.Config._
import tailcall.runtime.http.Method
import tailcall.runtime.remote.Remote
import tailcall.runtime.transcoder.Transcoder.Syntax
import zio.json.EncoderOps
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}

trait Config2Blueprint {

  implicit final private def jsonSchema: Schema[Json] =
    Schema[DynamicValue]
      .transformOrFail[Json](a => a.transcodeOrFailWith[Json, String], b => b.transcodeOrFailWith[DynamicValue, String])

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
          case _         => TSchema.`null`
        }
    }
  }

  final private def toEndpoint(config: Config, http: Step.Http, host: String): Endpoint =
    Endpoint.make(host).withPort(config.server.port.getOrElse(80)).withPath(http.path)
      .withMethod(http.method.getOrElse(Method.GET)).withInput(http.input).withOutput(http.output)

  final private def toRemoteMap(lookup: Remote[DynamicValue], map: Map[String, List[String]]): Remote[DynamicValue] =
    map.foldLeft(Remote(Map.empty[String, DynamicValue])) { case (to, (key, path)) =>
      lookup.path(path: _*).map(value => to.put(Remote(key), value)).getOrElse(to)
    }.toDynamic

  final private def toResolver(
    config: Config,
    steps: List[Step],
    field: Field
  ): Option[Remote[DynamicValue] => Remote[DynamicValue]] =
    steps match {
      case Nil => None

      case steps => config.server.host match {
          // TODO: should fail if Http is used without server.host
          case None if steps.exists(_.isInstanceOf[Step.Http]) => None
          case option                                          => option.map { host =>
              steps.map[Remote[DynamicValue] => Remote[DynamicValue]] {
                case http @ Step.Http(_, _, _, _) => input =>
                    val endpoint           = toEndpoint(config, http, host)
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

  final private def toDirective(step: List[Step]): Option[Blueprint.Directive] = {
    // TODO: should fail on error
    val (errors, jsons) = step.map(_.toJsonAST).partitionMap(identity(_))
    if (errors.nonEmpty || jsons.isEmpty) None
    else Json.Arr(jsons: _*).transcodeOrFailWith[DynamicValue, String] match {
      case Left(_)             => None
      case Right(dynamicValue) => Option(Blueprint.Directive(name = "steps", arguments = Map("value" -> dynamicValue)))
    }
  }

  final def toBlueprint(config: Config): TValid[Nothing, Blueprint] = {
    val rootSchema = Blueprint
      .SchemaDefinition(query = config.graphQL.schema.query, mutation = config.graphQL.schema.mutation)

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
            directives = toDirective(field.steps.getOrElse(Nil)).toList
          )
        }
      }

      Blueprint.ObjectTypeDefinition(name = name, fields = bFields)
    }

    TValid.succeed(Blueprint(rootSchema :: definitions))
  }
}
