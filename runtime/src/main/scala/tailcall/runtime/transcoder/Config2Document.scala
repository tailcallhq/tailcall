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
    val schema     = config.graphQL.schema
    val rootSchema = SchemaDefinition(
      query = schema.query,
      mutation = schema.mutation,
      subscription = None,
      directives = toServerDirective(config).toList,
    )

    val definitions: List[Definition] = getDefinitions(config)

    TValid.succeed(Document(rootSchema :: definitions, SourceMapper.empty))
  }

  private def getDefinitions(config: Config): List[Definition] = {
    config.graphQL.types.toList.map { case (name, typeInfo) =>
      val definition = toObjectTypeDefinition(name, typeInfo)
      if (typeInfo.isInput) toInputObjectTypeDefinition(definition) else definition
    }
  }

  final private def getName(typeOf: Type): String = {
    typeOf match {
      case NamedType(name, _)  => name
      case ListType(ofType, _) => getName(ofType)
    }
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

  final private def toFieldDefinition(typeInfo: Config.Type): List[FieldDefinition] = {
    typeInfo.fields.toList.map { case (name, field) =>
      val args: List[InputValueDefinition] = {
        field.args.getOrElse(Map.empty).toList.map { case (name, arg) =>
          val ofType     = toType(arg)
          val directives = arg.modify.toList.flatMap(_.toDirective.toList)

          InputValueDefinition(
            name = name,
            ofType = ofType,
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

  final private def toInputObjectTypeDefinition(definition: ObjectTypeDefinition): InputObjectTypeDefinition = {
    val fields = definition.fields.map { field =>
      InputValueDefinition(
        name = field.name,
        ofType = setName(field.ofType, getName(field.ofType)),
        defaultValue = None,
        description = field.description,

        // Dumb copy of directives, this is not always correct
        directives = field.directives,
      )
    }
    InputObjectTypeDefinition(
      name = definition.name,
      fields = fields,
      description = definition.description,
      directives = Nil,
    )
  }

  final private def toObjectTypeDefinition(name: String, typeInfo: Config.Type): ObjectTypeDefinition = {
    val fields: List[FieldDefinition] = toFieldDefinition(typeInfo)
    ObjectTypeDefinition(name = name, fields = fields, description = typeInfo.doc, implements = Nil, directives = Nil)
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
