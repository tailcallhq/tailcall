package tailcall.gateway.service

import caliban.GraphQL
import caliban.introspection.adt.__Directive
import caliban.schema.{Operation, RootSchemaBuilder}
import caliban.wrappers.Wrapper
import tailcall.gateway.ast.Document
import zio.{ZIO, ZLayer}

trait DocumentGraphQLGenerator {
  def toGraphQL(document: Document): GraphQL[Any]
}

object DocumentGraphQLGenerator {
  final case class Live(tGen: DocumentTypeGenerator, sGen: DocumentStepGenerator) extends DocumentGraphQLGenerator {
    override def toGraphQL(document: Document): GraphQL[Any] =
      new GraphQL[Any] {
        override protected val schemaBuilder: RootSchemaBuilder[Any]   = {
          val queryOperation = Operation(tGen.__type(document), sGen.resolve(document))
          RootSchemaBuilder(query = Option(queryOperation), None, None)
        }
        override protected val wrappers: List[Wrapper[Any]]            = Nil
        override protected val additionalDirectives: List[__Directive] = Nil
      }
  }

  def live: ZLayer[DocumentTypeGenerator with DocumentStepGenerator, Nothing, DocumentGraphQLGenerator] =
    ZLayer.fromFunction(Live.apply _)

  def toGraphQL(document: Document): ZIO[DocumentGraphQLGenerator, Nothing, GraphQL[Any]] =
    ZIO.serviceWith(_.toGraphQL(document))
}
