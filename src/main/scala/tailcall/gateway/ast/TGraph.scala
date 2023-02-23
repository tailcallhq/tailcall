package tailcall.gateway.ast

import caliban.GraphQL
import caliban.introspection.adt.{__Directive, __Type, __TypeKind}
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.wrappers.Wrapper
import tailcall.gateway.StepGenerator
import tailcall.gateway.ast.Orc.OrcObject
import tailcall.gateway.service.DynamicRuntime

final case class TGraph(orcs: List[Orc], query: Option[String] = None, mutation: Option[String] = None) {
  self =>
  def toGraphQL: GraphQL[DynamicRuntime] =
    new GraphQL[DynamicRuntime] {
      val schema = new caliban.schema.Schema[DynamicRuntime, TGraph] {
        override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
          __Type(__TypeKind.OBJECT)

        override def resolve(input: TGraph): Step[DynamicRuntime] = new StepGenerator(input).gen
      }

      override protected val schemaBuilder: RootSchemaBuilder[DynamicRuntime] =
        RootSchemaBuilder(Option(Operation(schema.toType_(), schema.resolve(self))), None, None, Nil, Nil)
      override protected val wrappers: List[Wrapper[Any]]                     = Nil
      override protected val additionalDirectives: List[__Directive]          = Nil
    }

  def withQuery(name: String): TGraph    = self.copy(query = Option(name))
  def withMutation(name: String): TGraph = self.copy(mutation = Option(name))
  def rootQuery: Option[String]          = query.orElse(orcs.headOption.collect { case OrcObject(name, _) => name })
  def rootMutation: Option[String]       = mutation.orElse(orcs.headOption.collect { case OrcObject(name, _) => name })
}

object TGraph {
  def apply(orc: Orc*): TGraph = TGraph(orc.toList)
  def empty: TGraph            = TGraph(Nil)
}
