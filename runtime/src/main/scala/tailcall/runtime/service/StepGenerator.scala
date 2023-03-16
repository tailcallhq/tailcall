package tailcall.runtime.service

import caliban.schema.Step
import tailcall.runtime.ast
import tailcall.runtime.ast.{Blueprint, Context}
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service.StepGenerator.StepResult
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

trait StepGenerator {
  def resolve(document: Blueprint): StepResult[HttpDataLoader]
}

object StepGenerator {
  def live: ZLayer[EvaluationRuntime, Nothing, StepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm =>
      new StepGenerator {
        override def resolve(document: Blueprint): StepResult[HttpDataLoader] =
          BlueprintGenerator(rtm, document).resolve
      }
    ))
  }

  def resolve(document: Blueprint): ZIO[StepGenerator, Nothing, StepResult[HttpDataLoader]] =
    ZIO.serviceWith(_.resolve(document))

  final case class StepResult[R](query: Option[Step[R]], mutation: Option[Step[R]])

  final case class BlueprintGenerator(rtm: EvaluationRuntime, document: Blueprint) {
    val rootContext: Context = Context(DynamicValue(()))

    val stepRef: Map[String, Context => Step[HttpDataLoader]] = document.definitions
      .collect { case obj @ Blueprint.ObjectTypeDefinition(_, _) => (obj.name, ctx => fromObjectDef(obj, ctx)) }.toMap

    def resolve: StepResult[HttpDataLoader] = {

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

    def fromFieldDefinition(field: Blueprint.FieldDefinition, ctx: Context): Step[HttpDataLoader] = {
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

    def fromObjectDef(obj: Blueprint.ObjectTypeDefinition, ctx: Context): Step[HttpDataLoader] = {
      Step.ObjectStep(obj.name, obj.fields.map(field => field.name -> fromFieldDefinition(field, ctx)).toMap)
    }

    def fromType(tpe: ast.Blueprint.Type, ctx: Context): Step[HttpDataLoader] =
      tpe match {
        case ast.Blueprint.NamedType(name, _)  => stepRef.get(name) match {
            case Some(value) => value(ctx)
            case None        => Step.PureStep(DynamicValueUtil.toResponseValue(ctx.value))
          }
        case ast.Blueprint.ListType(ofType, _) => ctx.value match {
            case DynamicValue.Sequence(values) => Step
                .ListStep(values.map(value => fromType(ofType, ctx.copy(value = value))).toList)
            case _                             => Step.ListStep(List(fromType(ofType, ctx)))
          }
      }
  }
}
