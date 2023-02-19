package tailcall.gateway.lambda

import tailcall.gateway.remote._
import zio.schema.{DeriveSchema, DynamicValue, Schema}

sealed trait ExecutionPlan

object ExecutionPlan {
  final case class Add(left: ExecutionPlan, right: ExecutionPlan)
      extends ExecutionPlan

  final case class Constant(value: DynamicValue, ctor: Constructor[Any])
      extends ExecutionPlan

  final case class Pipe(left: ExecutionPlan, right: ExecutionPlan)
      extends ExecutionPlan

  final case class FunctionDefinition(
    input: EvaluationContext.Key,
    body: ExecutionPlan
  ) extends ExecutionPlan

  final case class Lookup(key: EvaluationContext.Key) extends ExecutionPlan

  final case class Flatten(plan: ExecutionPlan) extends ExecutionPlan

  implicit val schema: Schema[ExecutionPlan] = DeriveSchema.gen[ExecutionPlan]
}
