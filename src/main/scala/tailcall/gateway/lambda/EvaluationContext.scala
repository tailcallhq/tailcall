package tailcall.gateway.lambda

import zio.{Ref, Task, ZIO, ZLayer}

trait EvaluationContext {
  def get(id: EvaluationContext.Key): Task[Any]
  def set(id: EvaluationContext.Key, value: Any): Task[Unit]
  def drop(id: EvaluationContext.Key): Task[Unit]
}

object EvaluationContext {
  final case class Key(level: Int, index: Int)

  object Key {
    def fromContext(ctx: CompilationContext): Key = Key(ctx.level, ctx.index)
  }

  final case class Default(map: Ref[Map[Key, Any]]) extends EvaluationContext {
    def get(id: Key): Task[Any] =
      map.get.flatMap { map =>
        map.get(id) match {
          case None        => ZIO.fail(EvaluationError.BindingNotFound(id))
          case Some(value) => ZIO.succeed(value)
        }
      }

    def set(id: Key, value: Any): Task[Unit] = map.update(_ + (id -> value))

    def drop(id: Key): Task[Unit] = map.update(_ - id)
  }

  def live: ZLayer[Any, Nothing, EvaluationContext] = ZLayer.fromZIO(Ref.make(Map.empty[Key, Any]).map(Default))

  def set(id: Key, value: Any): ZIO[EvaluationContext, EvaluationError, Unit] =
    ZIO.serviceWith[EvaluationContext](_.set(id, value))

  def get(id: Key): ZIO[EvaluationContext, EvaluationError, Any] = ZIO.serviceWith[EvaluationContext](_.get(id))

  def drop(id: Key): ZIO[EvaluationContext, EvaluationError, Unit] = ZIO.serviceWith[EvaluationContext](_.drop(id))
}
