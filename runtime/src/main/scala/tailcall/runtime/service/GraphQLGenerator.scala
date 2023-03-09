package tailcall.runtime.service

import caliban.GraphQL
import caliban.introspection.adt.__Directive
import caliban.schema.{Operation, RootSchemaBuilder}
import caliban.wrappers.Wrapper
import tailcall.runtime.ast.Blueprint
import zio.{ZIO, ZLayer}

trait GraphQLGenerator {
  def toGraphQL(document: Blueprint): GraphQL[Any]
}

object GraphQLGenerator {
  final case class Live(tGen: TypeGenerator, sGen: StepGenerator) extends GraphQLGenerator {
    override def toGraphQL(document: Blueprint): GraphQL[Any] =
      new GraphQL[Any] {
        override protected val schemaBuilder: RootSchemaBuilder[Any]   = {
          val queryOperation = for {
            __type  <- tGen.__type(document)
            resolve <- sGen.resolve(document)
          } yield Operation(__type, resolve)
          RootSchemaBuilder(query = queryOperation, None, None)
        }
        override protected val wrappers: List[Wrapper[Any]]            = Nil
        override protected val additionalDirectives: List[__Directive] = Nil
      }
  }

  def live: ZLayer[TypeGenerator with StepGenerator, Nothing, GraphQLGenerator] = ZLayer.fromFunction(Live.apply _)

  def toGraphQL(document: Blueprint): ZIO[GraphQLGenerator, Nothing, GraphQL[Any]] =
    ZIO.serviceWith(_.toGraphQL(document))
}
