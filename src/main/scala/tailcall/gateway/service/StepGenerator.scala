package tailcall.gateway.service

import caliban.schema.Step
import tailcall.gateway.ast
import tailcall.gateway.ast.Document.Resolver
import tailcall.gateway.ast.{Context, Document}
import tailcall.gateway.internal.DynamicValueUtil
import tailcall.gateway.remote.Remote
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

import scala.collection.mutable

trait StepGenerator {
  def resolve(document: Document): Option[Step[Any]]
}

object StepGenerator {
  final case class Live(rtm: EvaluationRuntime) extends StepGenerator {
    private val stepRef: mutable.Map[String, Step[Any]]     = mutable.Map.empty
    def resolve(field: Document.FieldDefinition): Step[Any] = {
      Step.FunctionStep { args =>
        val ctxArgs = args.view.mapValues(DynamicValueUtil.fromInputValue).toMap
        val context = Context(DynamicValue(()), ctxArgs, None)
        field.resolver match {
          case Resolver.FromFunction(f) => field.ofType match {
              case Document.NamedType(_, _)     => Step.QueryStep(ZQuery.fromZIO(
                  f(Remote(DynamicValue(context))).evaluate.map(DynamicValueUtil.toValue).map(Step.PureStep(_))
                    .provide(ZLayer.succeed(rtm))
                ))
              case Document.ListType(ofType, _) =>
                val resolver = f(Remote(DynamicValue(context))).evaluate.map { case DynamicValue.Sequence(values) =>
                  Step.ListStep(values.map(value => resolve(ofType)).toList)
                }.provide(ZLayer.succeed(rtm))

                Step.QueryStep(ZQuery.fromZIO(resolver))
            }
          case Resolver.Reference       => resolve(field.ofType)
        }
      }
    }

    def resolve(tpe: ast.Document.Type): Step[Any]             =
      tpe match {
        case ast.Document.NamedType(name, nonNull)  => stepRef.getOrElse(name, Step.NullStep)
        case ast.Document.ListType(ofType, nonNull) => Step.ListStep(List(resolve(ofType)))
      }
    def resolve(obj: Document.ObjectTypeDefinition): Step[Any] = {
      Step.ObjectStep(obj.name, obj.fields.map(field => field.name -> resolve(field)).toMap)
    }

    override def resolve(document: Document): Option[Step[Any]] = {
      document.definition.collect { case obj @ Document.ObjectTypeDefinition(_, _) =>
        stepRef.put(obj.name, resolve(obj))
      }

      for {
        query <- document.definition.collectFirst { case Document.SchemaDefinition(query, _, _) => query }.flatten
        step  <- stepRef.get(query)
      } yield step
    }
  }

  def live: ZLayer[EvaluationRuntime, Nothing, StepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Document): ZIO[StepGenerator, Nothing, Option[Step[Any]]] = ZIO.serviceWith(_.resolve(document))
}
