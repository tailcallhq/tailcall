package tailcall.gateway.service

import caliban.GraphQL
import caliban.introspection.adt.__Directive
import caliban.schema.{Operation, RootSchemaBuilder}
import caliban.wrappers.Wrapper
import tailcall.gateway.ast.Orchestration
import zio.{ZIO, ZLayer}

trait OrchestrationGraphQLGenerator {
  def toGraphQL(document: Orchestration): GraphQL[Any]
}

object OrchestrationGraphQLGenerator {
  final case class Live(tGen: OrchestrationTypeGenerator, sGen: OrchestrationStepGenerator)
      extends OrchestrationGraphQLGenerator {
    override def toGraphQL(document: Orchestration): GraphQL[Any] =
      new GraphQL[Any] {
        override protected val schemaBuilder: RootSchemaBuilder[Any]   = {
          val queryOperation = Operation(tGen.__type(document), sGen.resolve(document))
          RootSchemaBuilder(query = Option(queryOperation), None, None)
        }
        override protected val wrappers: List[Wrapper[Any]]            = Nil
        override protected val additionalDirectives: List[__Directive] = Nil
      }
  }

  def live: ZLayer[OrchestrationTypeGenerator with OrchestrationStepGenerator, Nothing, OrchestrationGraphQLGenerator] =
    ZLayer.fromFunction(Live.apply _)

  def toGraphQL(document: Orchestration): ZIO[OrchestrationGraphQLGenerator, Nothing, GraphQL[Any]] =
    ZIO.serviceWith(_.toGraphQL(document))
}
