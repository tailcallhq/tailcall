package tailcall.gateway.service

import zio.{Ref, UIO, ZIO, ZLayer}

trait EvaluationContext {
  def get(id: EvaluationContext.Binding): UIO[Option[Any]]
  def set(id: EvaluationContext.Binding, value: Any): UIO[Unit]
  def drop(id: EvaluationContext.Binding): UIO[Unit]
}

object EvaluationContext {
  final case class Binding(id: Int)

  final case class Live(map: Ref[Map[Binding, Any]]) extends EvaluationContext {
    def get(id: Binding): UIO[Option[Any]]      = map.get.map(_.get(id))
    def set(id: Binding, value: Any): UIO[Unit] = map.update(_ + (id -> value))
    def drop(id: Binding): UIO[Unit]            = map.update(_ - id)
  }

  def live: ZLayer[Any, Nothing, EvaluationContext] = ZLayer.fromZIO(Ref.make(Map.empty[Binding, Any]).map(Live))

  def set(id: Binding, value: Any): ZIO[EvaluationContext, Nothing, Unit] =
    ZIO.serviceWith[EvaluationContext](_.set(id, value))

  def get(id: Binding): ZIO[EvaluationContext, Nothing, Option[Any]] = ZIO.serviceWithZIO[EvaluationContext](_.get(id))

  def drop(id: Binding): ZIO[EvaluationContext, Nothing, Unit] = ZIO.serviceWith[EvaluationContext](_.drop(id))
}
