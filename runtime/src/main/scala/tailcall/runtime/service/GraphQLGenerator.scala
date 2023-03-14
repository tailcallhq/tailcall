package tailcall.runtime.service

import caliban.GraphQL
import caliban.introspection.adt.__Directive
import caliban.schema.{Operation, RootSchemaBuilder}
import caliban.wrappers.Wrapper
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.http.Request
import zio.{Chunk, ZIO, ZLayer}

trait GraphQLGenerator {
  def toGraphQL(document: Blueprint): GraphQL[DataLoader[Any, Throwable, Request, Chunk[Byte]]]
}

object GraphQLGenerator {
  final case class Live(tGen: SchemaGenerator, sGen: StepGenerator) extends GraphQLGenerator {
    override def toGraphQL(input: Blueprint): GraphQL[DataLoader[Any, Throwable, Request, Chunk[Byte]]] =
      new GraphQL[Any] {
        override protected val schemaBuilder: RootSchemaBuilder[Any]   = {
          val stepResult = sGen.resolve(input)

          val queryOperation = for {
            __type  <- tGen.__schema(input).map(_.queryType)
            resolve <- stepResult.query
          } yield Operation(__type, resolve)

          val mutationOperation = for {
            __type  <- tGen.__schema(input).flatMap(_.mutationType)
            resolve <- stepResult.mutation
          } yield Operation(__type, resolve)
          RootSchemaBuilder(query = queryOperation, mutationOperation, None)
        }
        override protected val wrappers: List[Wrapper[Any]]            = Nil
        override protected val additionalDirectives: List[__Directive] = Nil
      }
  }

  def live: ZLayer[SchemaGenerator with StepGenerator, Nothing, GraphQLGenerator] = ZLayer.fromFunction(Live.apply _)

  def toGraphQL(
    document: Blueprint
  ): ZIO[GraphQLGenerator, Nothing, GraphQL[DataLoader[Any, Throwable, Request, Chunk[Byte]]]] =
    ZIO.serviceWith(_.toGraphQL(document))
}
