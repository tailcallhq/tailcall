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
    private val stepRef: mutable.Map[String, Context => Step[Any]]        = mutable.Map.empty
    def resolve(field: Document.FieldDefinition, ctx: Context): Step[Any] = {
      Step.FunctionStep { args =>
        val ctxArgs = args.view.mapValues(DynamicValueUtil.fromInputValue).toMap
        val context = Context(ctx.value, ctxArgs, ctx.parent)
        field.resolver match {
          case Resolver.FromFunction(f) => Step.QueryStep(ZQuery.fromZIO(
              f(Remote(DynamicValue(context))).evaluate.flatMap(value =>
                field.ofType match {
                  case Document.NamedType(_, _) => ZIO.succeed(DynamicValueUtil.toValue(value)).map(Step.PureStep(_))
                  case Document.ListType(ofType, _) =>
                    val resolver = ZIO.succeed(value match {
                      case DynamicValue.Sequence(values) => Step
                          .ListStep(values.map(value => resolve(ofType, context.copy(value = value))).toList)
                    })

                    resolver

                }
              ).provide(ZLayer.succeed(rtm))
            ))

          case Resolver.Reference => resolve(field.ofType, context)
        }
      }
    }

    def resolve(tpe: ast.Document.Type, ctx: Context): Step[Any]             =
      tpe match {
        case ast.Document.NamedType(name, nonNull)  => stepRef.getOrElse(name, (_: Context) => Step.NullStep)(ctx)
        case ast.Document.ListType(ofType, nonNull) => Step.ListStep(List(resolve(ofType, ctx)))
      }
    def resolve(obj: Document.ObjectTypeDefinition, ctx: Context): Step[Any] = {
      Step.ObjectStep(obj.name, obj.fields.map(field => field.name -> resolve(field, ctx)).toMap)
    }

    override def resolve(document: Document): Option[Step[Any]] = {
      val rootContext = Context(DynamicValue(()))
      document.definition.collect { case obj @ Document.ObjectTypeDefinition(_, _) =>
        stepRef.put(obj.name, ctx => resolve(obj, ctx))
      }

      for {
        query <- document.definition.collectFirst { case Document.SchemaDefinition(query, _, _) => query }.flatten
        step  <- stepRef.get(query)
      } yield step(rootContext)
    }
  }

  def live: ZLayer[EvaluationRuntime, Nothing, StepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Document): ZIO[StepGenerator, Nothing, Option[Step[Any]]] = ZIO.serviceWith(_.resolve(document))
}
