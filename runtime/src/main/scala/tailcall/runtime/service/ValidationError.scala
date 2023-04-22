package tailcall.runtime.service

sealed trait ValidationError extends Throwable {
  self =>
  override def getMessage(): String = message
  def message: String
  def userInstructions: Option[String]
}

object ValidationError {
  final case class DecodingError(from: String, to: String, reason: String, howToFix: Option[String])
      extends ValidationError {
    override def message: String                  = s"Decoding error: $from -> $to: $reason"
    override def userInstructions: Option[String] = howToFix
  }
}
