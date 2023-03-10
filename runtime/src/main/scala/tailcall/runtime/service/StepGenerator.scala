package tailcall.runtime.service

import caliban.schema.Step
import tailcall.runtime.ast
import tailcall.runtime.ast.{Blueprint, Context}
import tailcall.runtime.internal.DynamicValueUtil
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

import scala.collection.mutable

trait StepGenerator {
  def resolve(document: Blueprint): Option[Step[Any]]
  def resolveMutation(document: Blueprint): Option[Step[Any]]
}

object StepGenerator {
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

    override def resolve(document: Blueprint): Option[Step[Any]] = {
      val rootContext = Context(DynamicValue(()))
      document.definitions.collect { case obj @ Blueprint.ObjectTypeDefinition(_, _) =>
        stepRef.put(obj.name, ctx => fromObjectDef(obj, ctx))
      }

      for {
        query <- document.schema.query
        step  <- stepRef.get(query)
      } yield step(rootContext)
    }

    override def resolveMutation(document: Blueprint): Option[Step[Any]] = {
      val rootContext = Context(DynamicValue(()))
      document.definitions.collect { case obj @ Blueprint.ObjectTypeDefinition(_, _) =>
        stepRef.put(obj.name, ctx => fromObjectDef(obj, ctx))
      }

      for {
        query <- document.schema.mutation
        step  <- stepRef.get(query)
      } yield step(rootContext)
    }
  }

  def live: ZLayer[EvaluationRuntime, Nothing, StepGenerator] = {
    ZLayer(ZIO.service[EvaluationRuntime].map(rtm => Live(rtm)))
  }

  def resolve(document: Blueprint): ZIO[StepGenerator, Nothing, Option[Step[Any]]] =
    ZIO.serviceWith(_.resolve(document))

  def resolveMutation(document: Blueprint): ZIO[StepGenerator, Nothing, Option[Step[Any]]] =
    ZIO.serviceWith(_.resolveMutation(document))
}
