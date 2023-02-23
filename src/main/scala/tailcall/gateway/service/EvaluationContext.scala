package tailcall.gateway.service

import tailcall.gateway.lambda.EvaluationError
import zio.{Ref, Task, ZIO, ZLayer}

trait EvaluationContext {
  def get(id: EvaluationContext.Binding): Task[Any]
  def set(id: EvaluationContext.Binding, value: Any): Task[Unit]
  def drop(id: EvaluationContext.Binding): Task[Unit]
}

object EvaluationContext {
  final case class Binding(id: Int)

  final case class Default(map: Ref[Map[Binding, Any]]) extends EvaluationContext {
    def get(id: Binding): Task[Any] =
      map.get.flatMap { map =>
        map.get(id) match {
          case None        => ZIO.fail(EvaluationError.BindingNotFound(id))
          case Some(value) => ZIO.succeed(value)
        }
      }

    def set(id: Binding, value: Any): Task[Unit] = map.update(_ + (id -> value))

    def drop(id: Binding): Task[Unit] = map.update(_ - id)
  }

  def live: ZLayer[Any, Nothing, EvaluationContext] = ZLayer.fromZIO(Ref.make(Map.empty[Binding, Any]).map(Default))

  def set(id: Binding, value: Any): ZIO[EvaluationContext, EvaluationError, Unit] =
    ZIO.serviceWith[EvaluationContext](_.set(id, value))

  def get(id: Binding): ZIO[EvaluationContext, EvaluationError, Any] = ZIO.serviceWith[EvaluationContext](_.get(id))

  def drop(id: Binding): ZIO[EvaluationContext, EvaluationError, Unit] = ZIO.serviceWith[EvaluationContext](_.drop(id))
}
