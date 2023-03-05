package tailcall.gateway.dsl.json

import tailcall.gateway.ast.{Blueprint, Endpoint}
import tailcall.gateway.http.Method
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}

// TODO: use ZLayer service pattern
final class ConfigBlueprint(config: Config) {

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

  def toEndpoint(http: Step.Http): Endpoint =
    Endpoint.make(config.server.host).withPort(config.server.port.getOrElse(80)).withPath(http.path)
      .withMethod(http.method.getOrElse(Method.GET)).withInput(http.input).withOutput(http.output)

  def toResolver(steps: List[Step]): Option[Resolver] =
    steps match {
      case Nil   => None
      case steps => Option {
          steps.map[Resolver] {
            case http @ Step.Http(_, _, _, _) => input => Remote.fromEndpoint(toEndpoint(http), input)
            case Step.Constant(json)          => _ => Remote(json).toDynamic
          }.reduce((a, b) => r => b(a(r)))
        }
    }

  def toBlueprint: Blueprint = {
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

          val resolver = toResolver(input.steps.getOrElse(Nil))

          Blueprint.FieldDefinition(name, args, ofType, resolver)
        }
      }

      Blueprint.ObjectTypeDefinition(name = name, fields = bFields)
    }

    Blueprint(rootSchema, definitions)
  }
}

object ConfigBlueprint {
  def make(config: Config): ConfigBlueprint = new ConfigBlueprint(config)
}
