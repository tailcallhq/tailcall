package tailcall.gateway.service

import caliban.schema.Step
import tailcall.gateway.ast.{Context, Orchestration}
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

trait OrchestrationStepGenerator {
  def resolve(document: Orchestration): Step[Any]
}

object OrchestrationStepGenerator {
  final case class Live(rtm: EvaluationRuntime) extends OrchestrationStepGenerator {
    def resolve(field: Orchestration.FieldDefinition): Step[Any] = {
      Step.QueryStep(ZQuery.fromZIO(
        field.resolver(Remote(Context(DynamicValue(())))).evaluate.map(DynamicValueUtil.toValue).map(Step.PureStep(_))
          .provide(ZLayer.succeed(rtm))
      ))
    }

    override def resolve(document: Orchestration): Step[Any] = {
      document.query match {
        case Some(Orchestration.ObjectTypeDefinition(name, fields)) => Step
            .ObjectStep(name, fields.map(field => field.name -> resolve(field)).toMap)
        case None                                                   => Step.NullStep
      }
    }
  }

  def live: ZLayer[EvaluationRuntime, Nothing, OrchestrationStepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Orchestration): ZIO[OrchestrationStepGenerator, Nothing, Step[Any]] =
    ZIO.serviceWith(_.resolve(document))
}
