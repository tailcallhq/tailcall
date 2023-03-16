package tailcall.cli.service

import tailcall.cli.CommandADT
import tailcall.runtime.service.ConfigReader
import zio.cli.HelpDoc.Span.{spans, strong, text, uri}
import zio.{Duration, ZIO, ZLayer}

trait CommandExecutor {
  def execute(command: CommandADT): ZIO[Any, Throwable, Unit]
}

object CommandExecutor {

  final class Live(log: Logger) extends CommandExecutor {
    def timed[R, E, A](program: ZIO[R, E, A]): ZIO[R, E, A] =
      for {
        start <- zio.Clock.nanoTime
        a     <- program
        end   <- zio.Clock.nanoTime
        _     <- log {
          val duration = Duration.fromNanos(end - start)
          text(s"Completed in ${duration.toMillis} ms.")
        }
      } yield a

    override def execute(command: CommandADT): ZIO[Any, Throwable, Unit] =
      timed {
        command match {
          case CommandADT.Deploy(digest, endpoint) => ZIO.attempt(println(s"Deploying $digest to $endpoint"))
          case CommandADT.Validate(file)           => for {
              _ <- log(spans(text("Running file validation on: "), uri(file.toAbsolutePath.toUri)))
              _ <- ConfigReader.config.readFile(file.toFile).tapError(log.error(_))
              _ <- log(strong("\uD83D\uDC4D File is valid and is ready to be deployed."))
            } yield ()
        }
      }
  }

  def execute(command: CommandADT): ZIO[CommandExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO[CommandExecutor](_.execute(command))

  def live: ZLayer[Logger, Nothing, CommandExecutor] = ZLayer.fromFunction(new Live(_))
}
