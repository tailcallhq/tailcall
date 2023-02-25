package tailcall.gateway.service

import caliban.schema.Step
import tailcall.gateway.ast.Graph
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

trait GraphStepGenerator {
  def resolve(graph: Graph): Step[Any]
}

object GraphStepGenerator {
  def live: ZLayer[EvaluationRuntime, Nothing, GraphStepGenerator] =
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => new Live(rtm)))

  def resolve(graph: Graph): ZIO[GraphStepGenerator, Nothing, Step[Any]] = ZIO.serviceWith(_.resolve(graph))

  final class Live(rtm: EvaluationRuntime) extends GraphStepGenerator {
    override def resolve(graph: Graph): Step[Any] = {
      Step.ObjectStep(
        "Query",
        graph.fields.map(fields =>
          fields.name -> {
            Step.QueryStep(ZQuery.fromZIO(
              rtm.evaluateAs[DynamicValue](Remote.fromRemoteFunction(fields.executable).compile)(DynamicValue {})
                .map(result => Step.PureStep(DynamicValueUtil.toValue(result)))
            ))
          }
        ).toMap
      )
    }
  }
}
