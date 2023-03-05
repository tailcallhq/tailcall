package tailcall.gateway.dsl.json

import tailcall.gateway.ast.Blueprint
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}

object ConfigBlueprint {
  import Config._

  type Resolver = Remote[DynamicValue] => Remote[DynamicValue]

  implicit def jsonSchema: Schema[Json] =
    Schema[DynamicValue].transform[Json](DynamicValueUtil.toJson, DynamicValueUtil.fromJson)

  def toType(field: Field): Blueprint.Type = {
    val ofType = Blueprint.NamedType(field.as, field.isRequired.getOrElse(false))
    val isList = field.isList.getOrElse(false)
    if (isList) Blueprint.ListType(ofType, false) else ofType
  }

  def toResolver(steps: List[Step]): Option[Resolver] =
    steps match {
      case Nil   => None
      case steps => Option {
          steps.map[Resolver] {
            case Step.Http(_, _, _, _) => _ => Remote.die("Http not implemented")
            case Step.Constant(json)   => _ => Remote(json).toDynamic
          }.reduce((a, b) => r => b(a(r)))
        }
    }

  def toBlueprint(config: Config): Blueprint = {
    val rootSchema = Blueprint
      .SchemaDefinition(query = config.graphQL.schema.query, mutation = config.graphQL.schema.mutation)

    val definitions: List[Blueprint.Definition] = config.graphQL.types.toList.map { case (name, fields) =>
      def bFields: List[Blueprint.FieldDefinition] = {
        fields.toList.map { case (name, input) =>
          val args: List[Blueprint.InputValueDefinition] = {
            input.args.getOrElse(Map.empty).toList.map { case (name, _) =>
              Blueprint.InputValueDefinition(name, toType(input), None)
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
