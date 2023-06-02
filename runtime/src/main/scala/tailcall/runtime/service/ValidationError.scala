package tailcall.runtime.service

import caliban.CalibanError
import tailcall.runtime.internal.TValid
import zio.{Chunk, ZIO}

/**
 * Use it only for errors that are produced by the user.
 */
sealed trait ValidationError extends Throwable {
  self =>
  override def getMessage: String = message
  def message: String

  def toZIO: ZIO[Any, ValidationError, Nothing] = ZIO.fail(self)

  def when(cond: => Boolean): ZIO[Any, ValidationError, Unit] = toZIO.when(cond).unit
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

  final case class GraphQLGenerationError(errors: CalibanError.ValidationError) extends ValidationError {
    override def message: String =
      s"""
         |GraphQL generation error:
         |
         |${errors.msg}: ${errors.explanatoryText}
         |""".stripMargin
  }

  /**
   * Error in the blueprint generation
   */
  final case class BlueprintGenerationError(errors: Chunk[TValid.Cause[String]]) extends ValidationError {
    override def message: String = {
      errors.map(e => s"  - ${e.trace.mkString("[", ", ", "]")}: ${e.message}").mkString(
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
