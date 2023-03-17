package tailcall.cli.service

import zio.cli.HelpDoc
import zio.cli.HelpDoc.Span
import zio.{Cause, UIO, ZIO, ZLayer}

trait Logger {
  self =>
  def log(doc: HelpDoc): UIO[Unit]
  def apply(doc: HelpDoc): UIO[Unit]     = log(doc)
  def apply(doc: Span): UIO[Unit]        = apply(doc)
  def apply(text: String): UIO[Unit]     = apply(Span.text(text))
  def error(error: Throwable): UIO[Unit] =
    ZIO.logErrorCause(Cause.fail(error)) *> apply(Span.Error(Span.Text(error.getMessage)))
}

object Logger {
  final class Live extends Logger {
    override def log(doc: HelpDoc): UIO[Unit] =
      ZIO.succeed(println(doc.toPlaintext(color = true, columnWidth = Int.MaxValue)))
  }

  def live: ZLayer[Any, Nothing, Logger] = ZLayer.succeed(new Live)
}
