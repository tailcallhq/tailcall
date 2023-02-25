package tailcall.gateway.service

import caliban.schema.Step
import tailcall.gateway.ast.Document
import zio.{ZIO, ZLayer}

trait DocumentStepGenerator {
  def resolve(document: Document): Step[Any]
}

object DocumentStepGenerator {
  final case class Live(rtm: EvaluationRuntime) extends DocumentStepGenerator {
    override def resolve(document: Document): Step[Any] = ???
  }

  def live: ZLayer[EvaluationRuntime, Nothing, DocumentStepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Document): ZIO[DocumentStepGenerator, Nothing, Step[Any]] = ZIO.serviceWith(_.resolve(document))
}
