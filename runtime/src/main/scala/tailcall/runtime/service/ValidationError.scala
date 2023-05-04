package tailcall.runtime.service

import zio.{Chunk, ZIO}

/**
 * Use it only for errors that are produced by the user.
 */
sealed trait ValidationError extends Throwable {
  self =>
  override def getMessage: String                             = message
  def message: String
  def when(cond: => Boolean): ZIO[Any, ValidationError, Unit] = toZIO.when(cond).unit
  def toZIO: ZIO[Any, ValidationError, Nothing]               = ZIO.fail(self)
}

object ValidationError {

  /**
   * Unexpected status code from a downstream service
   */
  final case class StatusCodeError(code: Int, url: String) extends ValidationError {
    override def message: String = s"Unexpected status code: $code url: ${url}"
  }

  /**
   * Decoding failure of some user input
   */
  final case class DecodingError(from: String, to: String, reason: String) extends ValidationError {
    override def message: String = s"Decoding error: $from -> $to: $reason"
  }

  /**
   * Error in the blueprint generation
   */
  final case class BlueprintGenerationError(errors: Chunk[String]) extends ValidationError {
    override def message: String = {
      errors.map(e => s"  - $e").mkString(
        """
          |Blueprint generation error:
          |
          |""".stripMargin,
        "\n",
        "\n",
      )
    }
  }
}
