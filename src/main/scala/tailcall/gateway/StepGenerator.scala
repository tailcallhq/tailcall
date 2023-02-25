package tailcall.gateway

import caliban.schema.Step
import tailcall.gateway.StepGenerator.RemoteStep
import tailcall.gateway.ast.{Context, Orc, TGraph}
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import tailcall.gateway.service.EvaluationRuntime
import zio.query.ZQuery
import zio.schema.DynamicValue

import scala.collection.mutable

final class StepGenerator(tGraph: TGraph) {
  private val orcMap: Map[String, Orc] = tGraph.orcs.collect { case orc @ Orc.OrcObject(name, _) => name -> orc }.toMap
  private val stepMap: mutable.Map[String, RemoteStep] = mutable.Map.empty

  def gen(orc: Orc, context: Remote[Context]): RemoteStep = {
    orc match {
      case Orc.OrcValue(dynamicValue)  => Step.PureStep(DynamicValueUtil.toValue(dynamicValue))
      case Orc.OrcObject(name, fields) => Step
          .ObjectStep(name, fields.map { case (name, orc) => (name, gen(orc, context)) })
      case Orc.OrcList(values)         => Step.ListStep(values.map(gen(_, context)))
      case Orc.OrcFunction(fun)        => Step.QueryStep(ZQuery.fromZIO(fun(context).evaluate.map(gen(_, context))))
      case Orc.OrcRef(name)            => stepMap.get(name) match {
          case Some(step) => step
          case None       => orcMap.get(name) match {
              case Some(orc) =>
                val step = Step.QueryStep(ZQuery.succeed(gen(orc, context)))
                stepMap.addOne(name -> step)
                step
              case None      => Step.NullStep
            }
        }
    }
  }

  def gen: RemoteStep = {
    val orc = tGraph.rootQuery.flatMap(orcMap.get).getOrElse(throw StepGenerator.QueryNotFound)
    gen(orc, Remote(Context(DynamicValue(()))))
  }
}

object StepGenerator {
  type RemoteStep = Step[EvaluationRuntime]

  sealed trait GraphQLSchemaGeneratorError extends Throwable
  case object QueryNotFound                extends GraphQLSchemaGeneratorError
}
