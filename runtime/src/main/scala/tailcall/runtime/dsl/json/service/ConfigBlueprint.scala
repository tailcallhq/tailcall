package tailcall.runtime.dsl.json.service

import tailcall.runtime.ast.{Blueprint, Endpoint}
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.http.Method
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.remote.Remote
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}
import zio.{ZIO, ZLayer}

trait ConfigBlueprint {
  def toBlueprint(config: Config): Blueprint
}

object ConfigBlueprint {
  final class Live extends ConfigBlueprint {

    import Config._

    type Resolver = Remote[DynamicValue] => Remote[DynamicValue]

    implicit def jsonSchema: Schema[Json] =
      Schema[DynamicValue].transform[Json](DynamicValueUtil.toJson, DynamicValueUtil.fromJson)

    def toType(field: Field): Blueprint.Type = {
      val ofType = Blueprint.NamedType(field.typeOf, field.isRequired.getOrElse(false))
      val isList = field.isList.getOrElse(false)
      if (isList) Blueprint.ListType(ofType, false) else ofType
    }

    def toType(inputType: Argument): Blueprint.Type = {
      val ofType = Blueprint.NamedType(inputType.typeOf, inputType.isRequired.getOrElse(false))
      val isList = inputType.isList.getOrElse(false)
      if (isList) Blueprint.ListType(ofType, false) else ofType
    }

    def toEndpoint(config: Config, http: Step.Http): Endpoint =
      Endpoint.make(config.server.host).withPort(config.server.port.getOrElse(80)).withPath(http.path)
        .withMethod(http.method.getOrElse(Method.GET)).withInput(http.input).withOutput(http.output)

    def toRemoteMap(lookup: Remote[DynamicValue], map: Map[String, List[String]]): Remote[DynamicValue] =
      map.foldLeft(Remote(Map.empty[String, DynamicValue])) { case (to, (key, path)) =>
        lookup.path(path: _*).map(value => to.put(Remote(key), value)).getOrElse(to)
      }.toDynamic

    def toResolver(config: Config, steps: List[Step]): Option[Resolver] =
      steps match {
        case Nil   => None
        case steps => Option {
            steps.map[Resolver] {
              case http @ Step.Http(_, _, _, _) => input => Remote.fromEndpoint(toEndpoint(config, http), input)
              case Step.Constant(json)          => _ => Remote(json).toDynamic
              case Step.ObjPath(map)            => input => toRemoteMap(input, map)
            }.reduce((a, b) => r => b(a(r)))
          }
      }

    def toBlueprint(config: Config): Blueprint = {
      val rootSchema = Blueprint
        .SchemaDefinition(query = config.graphQL.schema.query, mutation = config.graphQL.schema.mutation)

      val definitions: List[Blueprint.Definition] = config.graphQL.types.toList.map { case (name, fields) =>
        val bFields: List[Blueprint.FieldDefinition] = {
          fields.toList.map { case (name, input) =>
            val args: List[Blueprint.InputValueDefinition] = {
              input.args.getOrElse(Map.empty).toList.map { case (name, inputType) =>
                Blueprint.InputValueDefinition(name, toType(inputType), None)
              }
            }

            val ofType = toType(input)

            val resolver = toResolver(config, input.steps.getOrElse(Nil))

            Blueprint.FieldDefinition(name, args, ofType, resolver.map(Remote.toLambda(_)))
          }
        }

        Blueprint.ObjectTypeDefinition(name = name, fields = bFields)
      }

      Blueprint(rootSchema, definitions)
    }
  }

  def live: ZLayer[Any, Nothing, ConfigBlueprint] = ZLayer.succeed(new Live())
  def toBlueprint(config: Config)                 = ZIO.serviceWith[ConfigBlueprint](_.toBlueprint(config))
}
