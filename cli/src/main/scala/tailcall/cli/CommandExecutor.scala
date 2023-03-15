package tailcall.cli

import zio.{ZIO, ZLayer}

trait CommandExecutor {
  def execute(command: CommandADT): ZIO[Any, Throwable, Unit]
}

object CommandExecutor {
  final class Live extends CommandExecutor {
    override def execute(command: CommandADT): ZIO[Any, Throwable, Unit] =
      ZIO.attempt(println(s"Executing command: $command"))
  }

  def execute(command: CommandADT): ZIO[CommandExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO[CommandExecutor](_.execute(command))

  def live: ZLayer[Any, Nothing, CommandExecutor] = ZLayer.succeed(new Live)
}
