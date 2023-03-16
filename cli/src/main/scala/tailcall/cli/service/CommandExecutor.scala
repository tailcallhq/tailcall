package tailcall.cli.service

import tailcall.cli.CommandADT
import tailcall.runtime.service.ConfigReader
import zio.cli.HelpDoc
import zio.cli.HelpDoc.Span.{strong, text, uri}
import zio.{Duration, UIO, ZIO, ZLayer}

trait CommandExecutor {
  def execute(command: CommandADT): ZIO[Any, Throwable, Unit]
}

object CommandExecutor {
  import HelpDoc._

  def errorMessage(error: Throwable): UIO[Unit] = show(HelpDoc.p(Span.Error(Span.Text(error.getMessage))))
  def show(doc: HelpDoc): UIO[Unit] = ZIO.succeed(println(doc.toPlaintext(color = true, columnWidth = Int.MaxValue)))
  def show(span: Span): UIO[Unit]   = show(HelpDoc.p(span))
  def span(texts: Span*): Span      = Span.spans(texts.toList)
  def span(text: String): Span      = Span.Text(text)

  def timed[R, E, A](program: ZIO[R, E, A]): ZIO[R, E, A] =
    for {
      start <- zio.Clock.nanoTime
      a     <- program
      end   <- zio.Clock.nanoTime
      _     <- show {
        val duration = Duration.fromNanos(end - start)
        text(s"Completed in ${duration.toMillis} ms.")
      }
    } yield a

  final class Live extends CommandExecutor {
    override def execute(command: CommandADT): ZIO[Any, Throwable, Unit] =
      command match {
        case CommandADT.Deploy(digest, endpoint) => ZIO.attempt(println(s"Deploying $digest to $endpoint"))
        case CommandADT.Validate(file)           => timed {
            for {
              _ <- show(span(text("Running file validation on "), uri(file.toAbsolutePath.toUri)))
              _ <- ConfigReader.config.readFile(file.toFile).tapError(errorMessage(_))
              _ <- show(strong("File is valid and is ready to be deployed."))
            } yield ()
          }
      }
  }

  def execute(command: CommandADT): ZIO[CommandExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO[CommandExecutor](_.execute(command))

  def live: ZLayer[Any, Nothing, CommandExecutor] = ZLayer.succeed(new Live)
}
