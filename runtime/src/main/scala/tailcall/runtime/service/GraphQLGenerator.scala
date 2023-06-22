package tailcall.runtime.service

import caliban.GraphQL
import caliban.execution.Feature
import caliban.introspection.adt.__Directive
import caliban.schema.{Operation, RootSchemaBuilder}
import caliban.tools.RemoteSchema
import caliban.wrappers.Wrapper
import tailcall.runtime.model.Blueprint
import tailcall.runtime.transcoder.Transcoder
import zio.{ZIO, ZLayer}

trait GraphQLGenerator {
  def toGraphQL(document: Blueprint): GraphQL[HttpContext]
}

object GraphQLGenerator {
  final case class Live(sGen: StepGenerator) extends GraphQLGenerator {
    override def toGraphQL(blueprint: Blueprint): GraphQL[HttpContext] = {
      val schema = Transcoder.toDocument(blueprint).toOption.flatMap(RemoteSchema.parseRemoteSchema)
      new GraphQL[HttpContext] {
        override protected val schemaBuilder: RootSchemaBuilder[HttpContext] = {
          val stepResult = sGen.resolve(blueprint)

          val queryOperation = for {
            __type  <- schema.map(_.queryType)
            resolve <- stepResult.query
          } yield Operation(__type, resolve)

          val mutationOperation = for {
            __type  <- schema.flatMap(_.mutationType)
            resolve <- stepResult.mutation
          } yield Operation(__type, resolve)
          RootSchemaBuilder(query = queryOperation, mutationOperation, None)
        }
        override protected val wrappers: List[Wrapper[Any]]                  = Nil
        override protected val additionalDirectives: List[__Directive]       = Nil
        override protected val features: Set[Feature]                        = Set.empty
      }
    }
  }

  def live: ZLayer[StepGenerator, Nothing, GraphQLGenerator] = ZLayer.fromFunction(Live.apply _)

  def default: ZLayer[Any, Nothing, GraphQLGenerator] = StepGenerator.default >>> live

  def toGraphQL(document: Blueprint): ZIO[GraphQLGenerator, Nothing, GraphQL[HttpContext]] =
    ZIO.serviceWith(_.toGraphQL(document))
}
