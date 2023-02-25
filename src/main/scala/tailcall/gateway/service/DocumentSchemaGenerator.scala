package tailcall.gateway.service

import tailcall.gateway.ast.Document
import zio.{ZIO, ZLayer}

trait DocumentSchemaGenerator {
  def schema(document: Document): caliban.schema.Schema[Any, Document]
}

object DocumentSchemaGenerator {
  final case class Live(tpeGen: DocumentTypeGenerator, stepGen: DocumentStepGenerator) extends DocumentSchemaGenerator {
    override def schema(document: Document): caliban.schema.Schema[Any, Document] =
      new caliban.schema.Schema[Any, Document] {
        override protected[this] def toType(
          isInput: Boolean,
          isSubscription: Boolean
        ): caliban.introspection.adt.__Type = tpeGen.__type(document)

        override def resolve(input: Document): caliban.schema.Step[Any] = stepGen.resolve(input)
      }
  }

  def live: ZLayer[DocumentTypeGenerator with DocumentStepGenerator, Nothing, DocumentSchemaGenerator] = {
    ZLayer((ZIO.service[DocumentTypeGenerator] zip ZIO.service[DocumentStepGenerator]).map(i => Live(i._1, i._2)))
  }

  def schema(doc: Document): ZIO[DocumentSchemaGenerator, Nothing, caliban.schema.Schema[Any, Document]] =
    ZIO.service[DocumentSchemaGenerator].map(_.schema(doc))
}
