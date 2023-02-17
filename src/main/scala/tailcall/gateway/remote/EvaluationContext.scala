package tailcall.gateway.remote

import zio.{Ref, Task, ZIO, ZLayer}

trait EvaluationContext {
  def get(id: Int): Task[Any]
  def set(id: Int, value: Any): Task[Unit]
  def drop(id: Int): Task[Unit]
}

object EvaluationContext {
  final case class Default(map: Ref[Map[Int, Any]]) extends EvaluationContext {
    def get(id: Int): Task[Any] =
      map
        .get
        .flatMap { map =>
          map.get(id) match {
            case None        => ZIO.fail(EvaluationError.BindingNotFound(id))
            case Some(value) => ZIO.succeed(value)
          }
        }

    def set(id: Int, value: Any): Task[Unit] = map.update(_ + (id -> value))

    def drop(id: Int): Task[Unit] = map.update(_ - id)
  }

  def live: ZLayer[Any, Nothing, EvaluationContext] =
    ZLayer.fromZIO(Ref.make(Map.empty[Int, Any]).map(Default))

  def set(id: Int, value: Any): ZIO[EvaluationContext, EvaluationError, Unit] =
    ZIO.serviceWith[EvaluationContext](_.set(id, value))

  def get(id: Int): ZIO[EvaluationContext, EvaluationError, Any] =
    ZIO.serviceWith[EvaluationContext](_.get(id))

  def drop(id: Int): ZIO[EvaluationContext, EvaluationError, Unit] =
    ZIO.serviceWith[EvaluationContext](_.drop(id))
}
