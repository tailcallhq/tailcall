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
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Config.{Arg, Field}
import tailcall.runtime.model._

/**
 * This is used to generate a .graphQL file from a config.
 * Mostly used for testing and onboarding a new APIs.
 */
trait Config2Document {
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
      val fields: List[FieldDefinition] = {
        typeInfo.fields.toList.map { case (name, field) =>
          val args: List[InputValueDefinition] = {
            field.args.getOrElse(Map.empty).toList.map { case (name, arg) =>
              val ofType = toType(arg)

              val prefixedOfType: Type = inputTypeNames.get(getName(ofType)) match {
                case Some(name) => setName(ofType, name)
                case None       => ofType
              }

              val directives = arg.modify.toList.flatMap(_.toDirective.toList)
              InputValueDefinition(
                name = name,
                ofType = prefixedOfType,
                defaultValue = None,
                description = arg.doc,
                directives = directives,
              )
            }
          }

          val ofType     = toType(field)
          val directives = toDirective(field)
          FieldDefinition(name = name, args = args, ofType = ofType, directives = directives, description = field.doc)
        }
      }

      // NOTE: Should create a list of definitions
      // There should be an object type or a list of input object type
      val definition      = ObjectTypeDefinition(
        name = name,
        fields = fields,
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

  final private def toDirective(field: Config.Field): List[Directive] = {
    var directives = List.empty[Directive]
    if (field.http.nonEmpty) directives = directives ++ field.http.toList.flatMap(_.toDirective.toList)
    if (field.unsafeSteps.nonEmpty)
      directives = directives ++ field.unsafeSteps.flatMap(UnsafeSteps(_).toDirective.toOption).toList
    if (field.modify.nonEmpty) directives = directives ++ field.modify.toList.flatMap(_.toDirective.toList)
    if (field.inline.exists(_.path.nonEmpty))
      directives = directives ++ field.inline.flatMap(_.toDirective.toOption).toList
    directives
  }

  final private def toInputObjectTypeDefinition(
    definition: ObjectTypeDefinition,
    inputNames: Map[String, String],
  ): InputObjectTypeDefinition = {
    val fields = definition.fields.map { field =>
      InputValueDefinition(
        name = field.name,
        ofType = setName(field.ofType, inputNames.getOrElse(getName(field.ofType), getName(field.ofType))),
        defaultValue = None,
        description = field.description,

        // Dumb copy of directives, this is not always correct
        directives = field.directives,
      )
    }
    InputObjectTypeDefinition(
      name = inputNames.getOrElse(definition.name, definition.name),
      fields = fields,
      description = definition.description,
      directives = Nil,
    )
  }

  final private def toServerDirective(config: Config): Option[Directive] = {
    if (config.server.isEmpty) { None }
    else { config.server.toDirective.toOption }
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
