package tailcall.runtime.service

import caliban.Value
import caliban.schema.Step
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.model
import tailcall.runtime.model.{Blueprint, Context}
import tailcall.runtime.service.StepGenerator.StepResult
import tailcall.runtime.transcoder.Transcoder
import zio.query.ZQuery
import zio.schema.DynamicValue
import zio.{ZIO, ZLayer}

trait StepGenerator {
  def resolve(document: Blueprint): StepResult[HttpContext]
}

object StepGenerator {
  def default: ZLayer[Any, Nothing, StepGenerator] = EvaluationRuntime.default >>> live

  def live: ZLayer[EvaluationRuntime, Nothing, StepGenerator] = {
    ZLayer(for {
      rtm  <- ZIO.service[EvaluationRuntime]
      envs <- zio.System.envs.orElse(ZIO.succeed(Map.empty[String, String]))
    } yield new StepGenerator {
      override def resolve(document: Blueprint): StepResult[HttpContext] = Live(rtm, document, envs).resolve
    })

  }

  def resolve(document: Blueprint): ZIO[StepGenerator, Nothing, StepResult[HttpContext]] =
    ZIO.serviceWith(_.resolve(document))

  final case class StepResult[R](query: Option[Step[R]], mutation: Option[Step[R]])

  final private case class Live(rtm: EvaluationRuntime, document: Blueprint, env: Map[String, String]) {
    val rootContext: Context = Context(DynamicValue(()), env = env)

    // A map of all the object types and a way to construct an instance of them.
    val objectStepRef: Map[String, Context => Step[HttpContext]] = document.definitions
      .collect { case obj @ Blueprint.ObjectTypeDefinition(_, _, _) => (obj.name, ctx => fromObjectDef(obj, ctx)) }
      .toMap

    def resolve: StepResult[HttpContext] = {

      val queryStep = for {
        query <- document.schema.flatMap(_.query)
        qStep <- objectStepRef.get(query)
      } yield Step.QueryStep(ZQuery.fromZIO(ZIO.service[HttpContext].map(h => qStep(rootContext.copy(headers = h.headers.map(h => String.valueOf(h.key) -> String.valueOf(h.value)).toMap)))))

      val mutationStep = for {
        mutation <- document.schema.flatMap(_.mutation)
        mStep    <- objectStepRef.get(mutation)
      } yield Step.QueryStep(ZQuery.fromZIO(ZIO.service[HttpContext].map(h => mStep(rootContext.copy(headers = h.headers.map(h => String.valueOf(h.key) -> String.valueOf(h.value)).toMap)))))

      StepResult(queryStep, mutationStep)
    }

    def fromFieldDefinition(field: Blueprint.FieldDefinition, ctx: Context): Step[HttpContext] = {
      Step.FunctionStep { args =>
        val context = ctx
          .copy(args = args.view.mapValues(Transcoder.toDynamicValue(_).getOrElse(DynamicValue(()))).toMap)
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

    def fromObjectDef(obj: Blueprint.ObjectTypeDefinition, ctx: Context): Step[HttpContext] = {
      Step.ObjectStep(obj.name, obj.fields.map(field => field.name -> fromFieldDefinition(field, ctx)).toMap)
    }

    /**
     * This method converts create a step from a type. There
     * is an implicit assumption that the type and the
     * actual value, which is available in the ctx.value are
     * compatible. We bailout if the types are not
     * compatible with the value.
     */
    def fromType(tpe: model.Blueprint.Type, ctx: Context): Step[HttpContext] = {
      tpe match {
        case model.Blueprint.NamedType(name, _)        => objectStepRef.get(name) match {
            case Some(stepFunction) => stepFunction(ctx)
            // This is a case for scalar values
            case None               => Step.PureStep(Transcoder.toResponseValue(ctx.value).getOrElse(Value.NullValue))
          }
        case model.Blueprint.ListType(ofType, nonNull) =>
          val isNullable = !nonNull
          ctx.value match {
            // Value is guaranteed to be a seq, we should be able to type-assert it safely
            case DynamicValue.Sequence(values)                                       => Step
                .ListStep(values.toList.map(value => fromType(ofType, ctx.copy(value = value))))
            case DynamicValue.SomeValue(DynamicValue.Sequence(values)) if isNullable =>
              Step.ListStep(values.toList.map(value => fromType(ofType, ctx.copy(value = value))))
            case DynamicValue.NoneValue if isNullable                                => Step.PureStep(Value.NullValue)
            case _ => throw new RuntimeException(s"Unexpected value received for type ${tpe.render}")
          }
      }
    }
  }
}
