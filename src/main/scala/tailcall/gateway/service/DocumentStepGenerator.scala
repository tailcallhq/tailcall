package tailcall.gateway.service

import caliban.schema.Step
import tailcall.gateway.ast.Document.Definition.FieldDefinition
import tailcall.gateway.ast.{Context, Document}
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

trait DocumentStepGenerator {
  def resolve(document: Document): Step[Any]
}

object DocumentStepGenerator {
  final case class Live(rtm: EvaluationRuntime) extends DocumentStepGenerator {
    def resolve(field: FieldDefinition): Step[Any] = {
      Step.QueryStep(ZQuery.fromZIO(
        field.resolver.run(Remote(Context(DynamicValue(())))).evaluate.map(DynamicValueUtil.toValue)
          .map(Step.PureStep(_)).provide(ZLayer.succeed(rtm))
      ))
    }

    override def resolve(document: Document): Step[Any] = {
      document.query match {
        case Some(Document.Definition.ObjectTypeDefinition(name, fields)) => Step
            .ObjectStep(name, fields.map(field => field.name -> resolve(field)).toMap)
        case None                                                         => Step.NullStep
      }
    }
  }

  def live: ZLayer[EvaluationRuntime, Nothing, DocumentStepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Document): ZIO[DocumentStepGenerator, Nothing, Step[Any]] = ZIO.serviceWith(_.resolve(document))
}
