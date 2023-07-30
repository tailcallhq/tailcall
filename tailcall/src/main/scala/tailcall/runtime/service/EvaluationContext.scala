package tailcall.runtime.service

import tailcall.runtime.service.EvaluationContext.Binding

final case class EvaluationContext(map: Map[Binding, Any]) {
  def get(id: EvaluationContext.Binding): Option[Any]                   = map.get(id)
  def set(id: EvaluationContext.Binding, value: Any): EvaluationContext = EvaluationContext(map + (id -> value))
  def drop(id: EvaluationContext.Binding): EvaluationContext            = EvaluationContext(map - id)
}

object EvaluationContext {
  final case class Binding(id: Int)
  def make: EvaluationContext = EvaluationContext(Map.empty)
}
