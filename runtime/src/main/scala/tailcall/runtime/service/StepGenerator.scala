package tailcall.runtime.service

import caliban.schema.Step
import tailcall.runtime.ast
import tailcall.runtime.ast.{Blueprint, Context}
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.service.StepGenerator.StepResult
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

import scala.collection.mutable

trait StepGenerator {
  def resolve(document: Blueprint): StepResult[Any]
}

object StepGenerator {
  final case class StepResult[R](query: Option[Step[R]], mutation: Option[Step[R]])

  final case class Live(rtm: EvaluationRuntime) extends StepGenerator {
    private val stepRef: mutable.Map[String, Context => Step[Any]] = mutable.Map.empty

    def fromFieldDefinition(field: Blueprint.FieldDefinition, ctx: Context): Step[Any] = {
      Step.FunctionStep { args =>
        val context = ctx.copy(args = args.view.mapValues(DynamicValueUtil.fromInputValue).toMap)
        field.resolver match {
          case Some(resolver) =>
            val step = for {
              value <- rtm.evaluate(resolver)(DynamicValue(context))
              step = fromType(field.ofType, context.copy(value = value, parent = Option(ctx)))
            } yield step

            Step.QueryStep(ZQuery.fromZIO(step))
          case None           =>
            val value = DynamicValue(DynamicValueUtil.getPath(context.value, field.name :: Nil))
            fromType(field.ofType, context.copy(value = value))
        }
      }
    }

    def fromType(tpe: ast.Blueprint.Type, ctx: Context): Step[Any] =
      tpe match {
        case ast.Blueprint.NamedType(name, _)  => stepRef.get(name) match {
            case Some(value) => value(ctx)
            case None        => Step.PureStep(DynamicValueUtil.toValue(ctx.value))
          }
        case ast.Blueprint.ListType(ofType, _) => ctx.value match {
            case DynamicValue.Sequence(values) => Step
                .ListStep(values.map(value => fromType(ofType, ctx.copy(value = value))).toList)
            case _                             => Step.ListStep(List(fromType(ofType, ctx)))
          }
      }

    def fromObjectDef(obj: Blueprint.ObjectTypeDefinition, ctx: Context): Step[Any] = {
      Step.ObjectStep(obj.name, obj.fields.map(field => field.name -> fromFieldDefinition(field, ctx)).toMap)
    }

    override def resolve(document: Blueprint): StepResult[Any] = {
      val rootContext = Context(DynamicValue(()))
      document.definitions.collect { case obj @ Blueprint.ObjectTypeDefinition(_, _) =>
        stepRef.put(obj.name, ctx => fromObjectDef(obj, ctx))
      }

      val queryStep = for {
        query <- document.schema.query
        qStep <- stepRef.get(query)
      } yield qStep(rootContext)

      val mutationStep = for {
        mutation <- document.schema.mutation
        mStep    <- stepRef.get(mutation)
      } yield mStep(rootContext)

      StepResult(queryStep, mutationStep)
    }
  }

  def live: ZLayer[EvaluationRuntime, Nothing, StepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Blueprint): ZIO[StepGenerator, Nothing, StepResult[Any]] = ZIO.serviceWith(_.resolve(document))
}
