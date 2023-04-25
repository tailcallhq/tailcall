package tailcall.runtime.transcoder

import caliban.GraphQL
import caliban.introspection.adt.__Directive
import caliban.parsing.adt.Document
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.tools.RemoteSchema
import caliban.wrappers.Wrapper
import tailcall.runtime.internal.TValid

trait Document2SDL {
  final def toSDL(document: Document): TValid[Nothing, String] =
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
        override protected val additionalDirectives: List[__Directive] = Nil
      }.render
    }
}
