package tailcall.gateway.ast

import caliban.GraphQL
import caliban.introspection.adt.{__Directive, __Type, __TypeKind}
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.wrappers.Wrapper
import tailcall.gateway.StepGenerator
import tailcall.gateway.lambda.LambdaRuntime

final case class TGraph(orcs: List[Orc], query: Option[String] = None, mutation: Option[String] = None) {
  self =>
  def toGraphQL: GraphQL[LambdaRuntime] =
    new GraphQL[LambdaRuntime] {
      val schema = new caliban.schema.Schema[LambdaRuntime, TGraph] {
        override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
          __Type(__TypeKind.OBJECT)

        override def resolve(input: TGraph): Step[LambdaRuntime] = new StepGenerator(input).gen
      }

      override protected val schemaBuilder: RootSchemaBuilder[LambdaRuntime] =
        RootSchemaBuilder(Option(Operation(schema.toType_(), schema.resolve(self))), None, None, Nil, Nil)
      override protected val wrappers: List[Wrapper[Any]]                    = Nil
      override protected val additionalDirectives: List[__Directive]         = Nil
    }

  def withQuery(name: String): TGraph    = self.copy(query = Some(name))
  def withMutation(name: String): TGraph = self.copy(mutation = Some(name))
}

object TGraph {
  def apply(orc: Orc*): TGraph = TGraph(orc.toList)
  def empty: TGraph            = TGraph(Nil)
}
