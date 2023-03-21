package tailcall.cli.service

import zio.cli.HelpDoc
import zio.cli.HelpDoc.Span
import zio.{Cause, UIO, ZIO, ZLayer}

trait Logger {
  self =>
  def log(doc: HelpDoc): UIO[Unit]
  def apply(doc: HelpDoc): UIO[Unit]     = log(doc)
  def apply(doc: Span): UIO[Unit]        = log(HelpDoc.p(doc))
  def apply(text: String): UIO[Unit]     = log(HelpDoc.p(Span.text(text)))
  def error(error: Throwable): UIO[Unit] =
    ZIO.logErrorCause(Cause.fail(error)) *> log(HelpDoc.p(Span.Error(Span.Text(error.getMessage))))
}

object Logger {
  final class Live extends Logger {
    override def log(doc: HelpDoc): UIO[Unit] =
      ZIO.succeed(println(doc.toPlaintext(color = true, columnWidth = Int.MaxValue)))
  }

  def log(doc: HelpDoc): ZIO[Logger, Nothing, Unit]       = ZIO.serviceWithZIO(_.log(doc))
  def log(doc: Span): ZIO[Logger, Nothing, Unit]          = ZIO.serviceWithZIO(_(doc))
  def log(doc: String): ZIO[Logger, Nothing, Unit]        = ZIO.serviceWithZIO(_(doc))
  def error(error: Throwable): ZIO[Logger, Nothing, Unit] = ZIO.serviceWithZIO(_.error(error))

  def live: ZLayer[Any, Nothing, Logger] = ZLayer.succeed(new Live)
}
