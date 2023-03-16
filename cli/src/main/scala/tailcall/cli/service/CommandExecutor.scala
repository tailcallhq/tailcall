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
              _      <- log(spans(text("Running file validation on: "), uri(file.toAbsolutePath.toUri)))
              config <- ConfigReader.config.readFile(file.toFile).tapError(log.error(_))
              digest = config.toBlueprint.digest
              _ <- log(spans(text("Digest: "), strong(s"${digest.alg}:${digest.hex}")))
              _ <- log(strong(s"\uD83D\uDC4D File is valid."))
            } yield ()
        }
      }
  }

  def execute(command: CommandADT): ZIO[CommandExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO[CommandExecutor](_.execute(command))

  def live: ZLayer[Logger, Nothing, CommandExecutor] = ZLayer.fromFunction(new Live(_))
}
