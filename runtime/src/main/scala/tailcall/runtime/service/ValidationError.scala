package tailcall.runtime.service

import zio.ZIO

sealed trait ValidationError extends Throwable {
  self =>
  override def getMessage(): String                           = message
  def message: String
  def toZIO: ZIO[Any, ValidationError, Nothing]               = ZIO.fail(self)
  def when(cond: => Boolean): ZIO[Any, ValidationError, Unit] = toZIO.when(cond).unit
}

object ValidationError {
  final case class StatusCodeError(code: Int, url: String) extends ValidationError {
    override def message: String = s"Unexpected status code: $code url: ${url}"
  }

  final case class DecodingError(from: String, to: String, reason: String) extends ValidationError {
    override def message: String = s"Decoding error: $from -> $to: $reason"
  }
}
