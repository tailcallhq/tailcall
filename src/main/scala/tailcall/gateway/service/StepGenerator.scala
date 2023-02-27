package tailcall.gateway.service

import caliban.schema.Step
import tailcall.gateway.ast.{Context, Document}
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

trait StepGenerator {
  def resolve(document: Document): Step[Any]
}

object StepGenerator {
  final case class Live(rtm: EvaluationRuntime) extends StepGenerator {
    def resolve(field: Document.FieldDefinition): Step[Any] = {
      Step.FunctionStep { args =>
        val ctxArgs = args.view.mapValues(DynamicValueUtil.fromInputValue(_)).toMap
        val context = Context(DynamicValue(()), ctxArgs, None)
        Step.QueryStep(ZQuery.fromZIO(
          field.resolver(Remote.dynamic(context)).evaluate.map(DynamicValueUtil.toValue).map(Step.PureStep(_))
            .provide(ZLayer.succeed(rtm))
        ))
      }
    }

    override def resolve(document: Document): Step[Any] = {
      document.definition.collectFirst { case Document.SchemaDefinition(query, _, _) => query }.flatten.flatMap(name =>
        document.definition.collectFirst { case q @ Document.ObjectTypeDefinition(`name`, _) => q }.map {
          case Document.ObjectTypeDefinition(name, fields) => Step
              .ObjectStep(name, fields.map(field => field.name -> resolve(field)).toMap)
        }
      ).getOrElse(Step.NullStep)
    }
  }

  def live: ZLayer[EvaluationRuntime, Nothing, StepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Document): ZIO[StepGenerator, Nothing, Step[Any]] = ZIO.serviceWith(_.resolve(document))
}
