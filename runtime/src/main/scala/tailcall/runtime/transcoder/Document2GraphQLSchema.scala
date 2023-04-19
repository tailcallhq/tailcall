package tailcall.runtime.transcoder

import caliban.GraphQL
import caliban.introspection.adt._
import caliban.parsing.adt.Definition.TypeSystemDefinition.DirectiveLocation
import caliban.parsing.adt.Definition.TypeSystemDefinition.DirectiveLocation.{
  ExecutableDirectiveLocation,
  TypeSystemDirectiveLocation,
}
import caliban.parsing.adt.Definition.TypeSystemDefinition.TypeDefinition.InputValueDefinition
import caliban.parsing.adt.{Document, Type}
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.tools.RemoteSchema
import caliban.wrappers.Wrapper
import tailcall.runtime.internal.TValid

trait Document2GraphQLSchema {
  private def wrapNonNull(__type: __Type, nonNull: Boolean): __Type =
    if (nonNull) __Type(kind = __TypeKind.NON_NULL, ofType = Option(__type)) else __type

  private def toType(value: Type): __Type = {
    value match {
      case Type.NamedType(name, nonNull)  => __Type(
          kind = __TypeKind.OBJECT,
          name = Option(name),
          ofType = if (nonNull) Option(__Type(kind = __TypeKind.NON_NULL)) else None,
        )
      case Type.ListType(ofType, nonNull) =>
        __Type(kind = __TypeKind.LIST, ofType = Option(wrapNonNull(toType(ofType), nonNull)))
    }
  }

  private def toLocation(location: DirectiveLocation): __DirectiveLocation =
    location match {
      case location: DirectiveLocation.ExecutableDirectiveLocation => location match {
          case ExecutableDirectiveLocation.QUERY               => __DirectiveLocation.QUERY
          case ExecutableDirectiveLocation.MUTATION            => __DirectiveLocation.MUTATION
          case ExecutableDirectiveLocation.SUBSCRIPTION        => __DirectiveLocation.SUBSCRIPTION
          case ExecutableDirectiveLocation.FIELD               => __DirectiveLocation.FIELD
          case ExecutableDirectiveLocation.FRAGMENT_DEFINITION => __DirectiveLocation.FRAGMENT_DEFINITION
          case ExecutableDirectiveLocation.FRAGMENT_SPREAD     => __DirectiveLocation.FRAGMENT_SPREAD
          case ExecutableDirectiveLocation.INLINE_FRAGMENT     => __DirectiveLocation.INLINE_FRAGMENT
        }
      case location: DirectiveLocation.TypeSystemDirectiveLocation => location match {
          case TypeSystemDirectiveLocation.SCHEMA                 => __DirectiveLocation.SCHEMA
          case TypeSystemDirectiveLocation.SCALAR                 => __DirectiveLocation.SCALAR
          case TypeSystemDirectiveLocation.OBJECT                 => __DirectiveLocation.OBJECT
          case TypeSystemDirectiveLocation.FIELD_DEFINITION       => __DirectiveLocation.FIELD_DEFINITION
          case TypeSystemDirectiveLocation.ARGUMENT_DEFINITION    => __DirectiveLocation.ARGUMENT_DEFINITION
          case TypeSystemDirectiveLocation.INTERFACE              => __DirectiveLocation.INTERFACE
          case TypeSystemDirectiveLocation.UNION                  => __DirectiveLocation.UNION
          case TypeSystemDirectiveLocation.ENUM                   => __DirectiveLocation.ENUM
          case TypeSystemDirectiveLocation.ENUM_VALUE             => __DirectiveLocation.ENUM_VALUE
          case TypeSystemDirectiveLocation.INPUT_OBJECT           => __DirectiveLocation.INPUT_OBJECT
          case TypeSystemDirectiveLocation.INPUT_FIELD_DEFINITION => __DirectiveLocation.INPUT_FIELD_DEFINITION
          case TypeSystemDirectiveLocation.VARIABLE_DEFINITION    => __DirectiveLocation.VARIABLE_DEFINITION
        }
    }

  private def toInputValue(definition: InputValueDefinition): __InputValue =
    __InputValue(
      name = definition.name,
      description = definition.description,
      `type` = () => toType(definition.ofType),
      defaultValue = definition.defaultValue.map(_.toString),
    )
  final def toGraphQLSchema(document: Document): TValid[Nothing, String]   =
    TValid.succeed {
      new GraphQL[Any] {
        override protected val schemaBuilder: RootSchemaBuilder[Any]   = {
          val schema = RemoteSchema.parseRemoteSchema(document)
          RootSchemaBuilder(
            schema.map(_.queryType).map(__type => Operation(__type, Step.NullStep)),
            schema.flatMap(_.mutationType).map(__type => Operation(__type, Step.NullStep)),
            None,
            schemaDirectives = document.schemaDefinition.map(_.directives).getOrElse(Nil),
          )
        }
        override protected val wrappers: List[Wrapper[Any]]            = Nil
        override protected val additionalDirectives: List[__Directive] = document.directiveDefinitions
          .map { directive =>
            __Directive(
              name = directive.name,
              description = directive.description,
              locations = directive.locations.map(toLocation),
              args = directive.args.map(toInputValue),
            )
          }
      }.render
    }
}
