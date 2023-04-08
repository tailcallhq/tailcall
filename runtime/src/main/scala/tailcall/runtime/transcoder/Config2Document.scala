package tailcall.runtime.transcoder

import caliban.parsing.SourceMapper
import caliban.parsing.adt.Definition.TypeSystemDefinition.SchemaDefinition
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.{
  FieldDefinition,
  InputObjectTypeDefinition,
  InputValueDefinition,
  ObjectTypeDefinition,
}
import caliban.parsing.adt.Type.{ListType, NamedType}
import caliban.parsing.adt.{Definition, Directive, Document, Type}
import tailcall.runtime.DirectiveCodec.EncoderSyntax
import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.internal.{JsonSchema, TValid}
import tailcall.runtime.model.Config.{Arg, Field}
import tailcall.runtime.model._
import tailcall.runtime.remote.Remote
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}

trait Config2Document {

  implicit final private def jsonSchema: Schema[Json] = JsonSchema.schema

  /**
   * Encodes a config into a Document
   */
  final def toDocument(config: Config): TValid[Nothing, Document] = {
    val rootSchema = SchemaDefinition(
      query = config.graphQL.schema.query,
      mutation = config.graphQL.schema.mutation,
      subscription = None,
      directives = toServerDirective(config).toList,
    )

    val outputTypes    = getOutputTypes(config).toSet
    val inputTypes     = getInputTypes(config).toSet
    val inputTypeNames = inputTypes.map { name =>
      if (outputTypes.contains(name)) name -> (name + "Input") else name -> name
    }.toMap

    val definitions: List[Definition] = config.graphQL.types.toList.flatMap { case (name, typeInfo) =>
      val bFields: List[FieldDefinition] = {
        typeInfo.fields.toList.map { case (name, field) =>
          val args: List[InputValueDefinition] = {
            field.args.getOrElse(Map.empty).toList.map { case (name, arg) =>
              val ofType = toType(arg)

              val prefixedOfType: Type = inputTypeNames.get(getName(ofType)) match {
                case Some(name) => setName(ofType, name)
                case None       => ofType
              }
              InputValueDefinition(
                name = name,
                ofType = prefixedOfType,
                defaultValue = None,
                description = arg.doc,
                directives = Nil,
              )
            }
          }

          val ofType     = toType(field)
          // val resolver   = toResolver(config, field.steps.getOrElse(Nil), field)
          val directives = toDirective(field.steps.getOrElse(Nil)).toList

          FieldDefinition(name = name, args = args, ofType = ofType, directives = directives, description = field.doc)
        }
      }

      // NOTE: Should create a list of definitions
      // There should be an object type or a list of input object type
      val definition      = ObjectTypeDefinition(
        name = name,
        fields = bFields,
        description = typeInfo.doc,
        implements = Nil,
        directives = Nil,
      )
      val inputDefinition = toInputObjectTypeDefinition(definition, inputTypeNames)
      if (outputTypes.contains(name) && inputTypes.contains(name)) List(definition, inputDefinition)
      else if (inputTypes.contains(name)) inputDefinition :: Nil
      else definition :: Nil
    }

    TValid.succeed(Document(rootSchema :: definitions, SourceMapper.empty))
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

  final private def getName(typeOf: Type): String = {
    typeOf match {
      case NamedType(name, _)  => name
      case ListType(ofType, _) => getName(ofType)
    }
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

  final private def setName(typeOf: Type, name: String): Type = {
    typeOf match {
      case NamedType(_, isRequired)  => NamedType(name, isRequired)
      case ListType(ofType, nonNull) => ListType(setName(ofType, name), nonNull)
    }
  }

  final private def toDirective(steps: List[Step]): Option[Directive] = {
    if (steps.isEmpty) None else steps.toDirective.toOption
  }

  final private def toEndpoint(http: Step.Http, host: String, port: Int): Endpoint = {
    Endpoint.make(host).withPort(port).withPath(http.path).withProtocol(if (port == 443) Scheme.Https else Scheme.Http)
      .withMethod(http.method.getOrElse(Method.GET)).withInput(http.input).withOutput(http.output)
  }

  private def toInputObjectTypeDefinition(
    definition: ObjectTypeDefinition,
    inputNames: Map[String, String],
  ): InputObjectTypeDefinition = {
    val fields = definition.fields.map { field =>
      InputValueDefinition(
        name = field.name,
        ofType = setName(field.ofType, inputNames.getOrElse(getName(field.ofType), getName(field.ofType))),
        defaultValue = None,
        description = field.description,
        directives = Nil,
      )
    }
    InputObjectTypeDefinition(
      name = inputNames.getOrElse(definition.name, definition.name),
      fields = fields,
      description = definition.description,
      directives = Nil,
    )
  }

  final private def toRemoteMap(lookup: Remote[DynamicValue], map: Map[String, List[String]]): Remote[DynamicValue] =
    map.foldLeft(Remote(Map.empty[String, DynamicValue])) { case (to, (key, path)) =>
      lookup.path(path: _*).map(value => to.put(Remote(key), value)).getOrElse(to)
    }.toDynamic

  // FIXME: should use this to generate steps
  final def toResolver(
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

  final private def toServerDirective(config: Config): Option[Directive] = {
    if (config.server.isEmpty) { None }
    else { config.server.toDirective.toOption }
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

  final private def toType(inputType: Arg): Type = {
    val ofType = NamedType(inputType.typeOf, inputType.isRequired)
    val isList = inputType.isList
    if (isList) ListType(ofType, false) else ofType
  }

  final private def toType(field: Field): Type = {
    val ofType = NamedType(field.typeOf, field.isRequired)
    val isList = field.isList
    if (isList) ListType(ofType, false) else ofType
  }
}
