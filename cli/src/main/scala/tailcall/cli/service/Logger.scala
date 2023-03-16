package tailcall.cli.service

import zio.cli.HelpDoc
import zio.cli.HelpDoc.Span
import zio.{Cause, UIO, ZIO, ZLayer}

trait Logger {
  def log(doc: => Span): UIO[Unit]
  def apply(doc: => Span): UIO[Unit]        = log(doc)
  def error(error: => Throwable): UIO[Unit] = {
    ZIO.logErrorCause(Cause.fail(error)) *> log(Span.Error(Span.Text(error.getMessage)))
  }
}

object Logger {
  final class Live extends Logger {
    override def log(doc: => Span): UIO[Unit] =
      ZIO.succeed(println(HelpDoc.p(doc).toPlaintext(color = true, columnWidth = Int.MaxValue)))
  }

  def log(doc: => Span): ZIO[Logger, Nothing, Unit] = ZIO.serviceWithZIO[Logger](_.log(doc))
  def live: ZLayer[Any, Nothing, Logger]            = ZLayer.succeed(new Live)
}
